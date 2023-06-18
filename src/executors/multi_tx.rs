use anyhow::Result;
use artemis_core::{
    executors::mempool_executor::{MempoolExecutor, SubmitTxToMempool},
    types::Executor,
};
use async_trait::async_trait;
use ethers::prelude::*;
use std::sync::Arc;
use tokio::task::JoinSet;

pub struct MultiMempoolExecutor<M>(Arc<MempoolExecutor<M>>);

pub type MultiSubmitTx = Vec<SubmitTxToMempool>;

impl<M: Middleware> MultiMempoolExecutor<M> {
    pub fn new(client: Arc<M>) -> Self {
        Self(Arc::new(MempoolExecutor::new(client)))
    }
}

#[async_trait]
impl<M> Executor<MultiSubmitTx> for MultiMempoolExecutor<M>
where
    M: Middleware + 'static,
    M::Error: 'static,
{
    async fn execute(&self, action: MultiSubmitTx) -> Result<()> {
        let mut set = JoinSet::new();

        action.into_iter().for_each(|tx| {
            let executor = Arc::clone(&self.0);
            set.spawn(async move { executor.execute(tx).await.unwrap_or_default() });
        });

        while let Some(_) = set.join_next().await {}

        Ok(())
    }
}
