use anyhow::Result;
use artemis_core::{
    executors::flashbots_executor::{FlashbotsBundle, FlashbotsExecutor},
    types::Executor,
};
use async_trait::async_trait;
use ethers::prelude::*;
use std::sync::Arc;
use tokio::task::JoinSet;
use url::Url;

pub struct MultiFlashbotsExecutor<M, S>(Arc<FlashbotsExecutor<M, S>>);

pub type MultiFlashbotsBundle = Vec<FlashbotsBundle>;

impl<M: Middleware, S: Signer> MultiFlashbotsExecutor<M, S> {
    pub fn new(client: Arc<M>, tx_signer: S, relay_signer: S, relay_url: impl Into<Url>) -> Self {
        Self(Arc::new(FlashbotsExecutor::new(
            client,
            tx_signer,
            relay_signer,
            relay_url,
        )))
    }
}

#[async_trait]
impl<M, S> Executor<MultiFlashbotsBundle> for MultiFlashbotsExecutor<M, S>
where
    M: Middleware + 'static,
    M::Error: 'static,
    S: Signer + 'static,
{
    async fn execute(&self, action: MultiFlashbotsBundle) -> Result<()> {
        let mut set = JoinSet::new();

        action.into_iter().for_each(|bundle| {
            let executor = Arc::clone(&self.0);
            set.spawn(async move { executor.execute(bundle).await.unwrap_or_default() });
        });

        while let Some(_) = set.join_next().await {}

        Ok(())
    }
}
