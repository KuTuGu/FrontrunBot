use anyhow::{anyhow, Result};
use ethers::prelude::*;
use ethers::types::transaction::eip2718::TypedTransaction;
use std::ops::Deref;
use std::sync::Arc;

abigen!(IArbitrage, "out/Arbitrage.sol/Arbitrage.json");

pub struct ArbitrageContract {
    inner: IArbitrage<Arc<Provider<Http>>>,
    signer: Address,
    client: Arc<Provider<Http>>,
}

impl Deref for ArbitrageContract {
    type Target = IArbitrage<Arc<Provider<Http>>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl ArbitrageContract {
    pub fn init(client: Arc<Provider<Http>>, signer: Address, address: Address) -> Self {
        Self {
            inner: IArbitrage::new(address, Arc::new(Arc::clone(&client))),
            signer,
            client,
        }
    }

    pub async fn to_tx<T: Into<TypedTransaction>>(
        &self,
        tx_list: Vec<T>,
        parent_block: Option<BlockNumber>,
        priority: Option<U256>,
    ) -> Result<TypedTransaction> {
        Ok(self
            .run(self.parse_tx_list(tx_list, parent_block, priority).await?)
            .from(self.signer)
            .tx)
    }

    async fn parse_tx_list<T: Into<TypedTransaction>>(
        &self,
        tx_list: Vec<T>,
        parent_block: Option<BlockNumber>,
        priority: Option<U256>,
    ) -> Result<Bytes> {
        let mut call_list = Vec::new();
        for tx in tx_list {
            let tx: TypedTransaction = tx.into();
            call_list.push(abi::Token::Bytes(abi::encode(&[
                abi::Token::Address(*tx.to_addr().unwrap_or(&Address::zero())),
                abi::Token::Uint(*tx.value().unwrap_or(&U256::zero())),
                abi::Token::Bytes(tx.data().unwrap_or(&Bytes::from(vec![0])).to_vec()),
            ])));
        }

        let block_hash = if parent_block.is_some() {
            let block = self
                .client
                .get_block(parent_block.unwrap())
                .await?
                .ok_or(anyhow!("Get block info error"))?;
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
