use std::sync::Arc;

use anyhow::Result;
use artemis_core::types::{Collector, CollectorStream};
use async_trait::async_trait;
use ethers::prelude::*;

/// A collector simply returns the transaction data for testing
pub struct EchoCollector<M> {
    provider: Arc<M>,
    tx: TxHash,
}

impl<M> EchoCollector<M> {
    pub fn new(provider: Arc<M>, tx: TxHash) -> Self {
        Self { provider, tx }
    }
}

#[async_trait]
impl<M> Collector<Transaction> for EchoCollector<M>
where
    M: Middleware,
    M::Provider: PubsubClient,
    M::Error: 'static,
{
    async fn get_event_stream(&self) -> Result<CollectorStream<'_, Transaction>> {
        let stream = futures_util::stream::iter(vec![self.tx]);
        let stream = TransactionStream::new(self.provider.provider(), stream, 1);
        let stream = stream.filter_map(|res| async move { res.ok() });
        Ok(Box::pin(stream))
    }
}
