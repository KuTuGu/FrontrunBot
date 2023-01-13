use ethers::prelude::*;
use ethers::types::transaction::eip2718::TypedTransaction;
use std::error::Error;
use std::ops::Deref;
use std::sync::Arc;

abigen!(Arbitrage, "out/Arbitrage.sol/Arbitrage.json");

pub struct ArbitrageUtil<'a, M, S> {
    inner: Arbitrage<&'a SignerMiddleware<M, S>>,
    client: &'a SignerMiddleware<M, S>,
}

impl<'a, M, S> Deref for ArbitrageUtil<'a, M, S> {
    type Target = Arbitrage<&'a SignerMiddleware<M, S>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, M: Middleware, S: Signer> ArbitrageUtil<'a, M, S> {
    pub fn init(client: &'a SignerMiddleware<M, S>, contract: Address) -> Self {
        Self {
            inner: Arbitrage::new(contract, Arc::new(client)),
            client,
        }
    }

    pub async fn deploy(
        client: &'a SignerMiddleware<M, S>,
    ) -> Result<ArbitrageUtil<'a, M, S>, Box<dyn Error + 'a>> {
        Ok(Self {
            inner: Arbitrage::deploy(Arc::new(client.clone()), ())
                .unwrap()
                .send()
                .await?,
            client,
        })
    }

    pub async fn to_tx<T: Into<TypedTransaction>>(
        &self,
        transaction_list: Vec<T>,
        uncle_protect: bool,
        priority: Option<U256>,
    ) -> Result<TypedTransaction, Box<dyn Error + 'a>> {
        Ok(self
            .run(
                self.parse_transaction_list(transaction_list, uncle_protect, priority)
                    .await?,
            )
            .from(self.client().address())
            .tx)
    }

    async fn parse_transaction_list<T: Into<TypedTransaction>>(
        &self,
        transaction_list: Vec<T>,
        uncle_protect: bool,
        priority: Option<U256>,
    ) -> Result<Bytes, Box<dyn Error + 'a>> {
        let mut call_list = Vec::new();
        for transaction in transaction_list {
            let transaction: TypedTransaction = transaction.into();
            call_list.push(abi::Token::Bytes(abi::encode(&[
                abi::Token::Address(*transaction.to_addr().unwrap_or(&Address::zero())),
                abi::Token::Uint(*transaction.value().unwrap_or(&U256::zero())),
                abi::Token::Bytes(transaction.data().unwrap_or(&Bytes::from(vec![0])).to_vec()),
            ])));
        }

        let block_hash = if uncle_protect {
            let last_block_number = self.client.get_block_number().await?;
            let block = self.client.get_block(last_block_number).await?.unwrap();
            block.hash.unwrap()
        } else {
            TxHash::zero()
        };
        Ok(abi::encode(&[
            abi::Token::FixedBytes((*block_hash.as_fixed_bytes()).into()),
            abi::Token::Uint(priority.unwrap_or_default()),
            abi::Token::Array(call_list),
        ])
        .into())
    }
}
