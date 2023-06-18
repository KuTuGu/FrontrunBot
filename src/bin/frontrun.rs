use arbitrage::{
    executors::multi_bundle::MultiFlashbotsExecutor, strategies::frontrun::FrontrunStrategy,
    utils::*,
};
use artemis_core::{collectors::mempool_collector::MempoolCollector, engine::Engine};
use dotenv::dotenv;
use ethers::prelude::*;
use std::sync::Arc;
use url::Url;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let http_url = get_env("HTTP_RPC_URL");
    let wss_url = get_env("WSS_RPC_URL");
    let relay_url = get_env("RELAY_URL").parse::<Url>().unwrap();
    let chain_id = get_env("CHAIN_ID").parse::<u16>().unwrap_or(1);
    let priority = get_env("PRIORITY").parse::<u64>().ok();
    let contract = get_env("CONTRACT").parse::<Address>().ok();
    let private_key = get_env("PRIVATE_KEY").replace("0x", "");

    let http_provider = Arc::new(Provider::<Http>::connect(&http_url).await);
    let wss_provider = Arc::new(
        Provider::<Ws>::connect(wss_url)
            .await
            .expect("Websocket connect error"),
    );
    let wallet = private_key
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(chain_id);

    let mut engine = Engine::new();
    engine.add_collector(Box::new(MempoolCollector::new(Arc::clone(&wss_provider))));
    engine.add_strategy(Box::new(FrontrunStrategy::new(
        Arc::clone(&http_provider),
        wallet.address(),
        contract,
        priority,
        true,
    )));
    engine.add_executor(Box::new(MultiFlashbotsExecutor::new(
        Arc::clone(&http_provider),
        wallet,
        LocalWallet::new(&mut rand::thread_rng()),
        relay_url,
    )));
    let mut set = engine.run().await.unwrap();
    while let Some(_) = set.join_next().await {}
}
