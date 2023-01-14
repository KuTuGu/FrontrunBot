use ethers::core::rand::thread_rng;
use ethers::prelude::*;
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers_flashbots::*;
use std::error::Error;
use std::ops::Deref;
use url::Url;

type Singer = SignerMiddleware<FlashbotsMiddleware<Provider<Http>, LocalWallet>, LocalWallet>;

pub struct FlashBotUtil {
    pub inner: Singer,
}

impl Deref for FlashBotUtil {
    type Target = Singer;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl FlashBotUtil {
    pub fn init(provider: Provider<Http>, wallet: LocalWallet) -> Option<Self> {
        if let Some(endpoint) = match wallet.chain_id() {
            1 => Some("https://relay.flashbots.net"),
            5 => Some("https://relay-goerli.flashbots.net"),
            _ => None,
        } {
            let flashbot = SignerMiddleware::new(
                FlashbotsMiddleware::new(
                    provider,
                    Url::parse(endpoint).unwrap(),
                    LocalWallet::new(&mut thread_rng()),
                ),
                wallet,
            );
            return Some(Self { inner: flashbot });
        }

        None
    }

    pub async fn run<T: Into<TypedTransaction>>(
        &self,
        tx_list: Vec<T>,
    ) -> Result<TxHash, Box<dyn Error>> {
        let bundle = self.to_bundle(tx_list).await?;
        self.inner().simulate_bundle(&bundle).await?;
        Ok(self.inner().send_bundle(&bundle).await?.await?)
    }

    async fn to_bundle<T: Into<TypedTransaction>>(
        &self,
        tx_list: Vec<T>,
    ) -> Result<BundleRequest, Box<dyn Error>> {
        let last_block_number = self.get_block_number().await?;
        let mut bundle = BundleRequest::new()
            .set_block(last_block_number + 1)
            .set_simulation_block(last_block_number)
            .set_simulation_timestamp(0);

        for tx in tx_list {
            let mut tx: TypedTransaction = tx.into();
            self.fill_transaction(&mut tx, None).await?;
            let signature = self.signer().sign_transaction(&tx).await?;
            bundle = bundle.push_transaction(tx.rlp_signed(&signature));
        }

        Ok(bundle)
    }
}
