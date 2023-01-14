use ethers::prelude::*;
use std::future::Future;

pub struct ListenPool {
    wss_provider: Provider<Ws>,
    max_concurrent: Option<usize>,
}

impl ListenPool {
    pub async fn init(wss_url: &str, max_concurrent: Option<usize>) -> Self {
        Self {
            wss_provider: Provider::<Ws>::connect(wss_url)
                .await
                .expect("Websocket connect error"),
            max_concurrent,
        }
    }

    pub async fn run<Fut: Future<Output = ()>, F: FnMut(TxHash) -> Fut>(&self, handle: F) {
        self.wss_provider
            .subscribe_pending_txs()
            .await
            .expect("Subscribe pending txs error")
            .for_each_concurrent(self.max_concurrent, handle)
            .await;
    }
}
