use arbitrage::{
    collectors::echo::EchoCollector, executors::multi_tx::MultiMempoolExecutor,
    strategies::frontrun::FrontrunStrategy, utils::*,
};
use artemis_core::types::{Collector, Executor, Strategy};
use ethers::{prelude::*, utils::Anvil};
use std::sync::Arc;

#[tokio::test]
async fn frontrun_test() {
    const HTTP_RPC_URL: &str = "https://rpc.ankr.com/eth";
    const CHAIN_ID: u64 = 1;
    const BLOCK_NUMBER: u64 = 16298449;
    const TX_HASH: &str = "0x12d867ee837cec251b067319e2802c15b01dc2e18b052b95fcd6657e19ff2a5e";

    let anvil = Anvil::new()
        .chain_id(CHAIN_ID)
        .port(8545_u16)
        .fork(HTTP_RPC_URL)
        .fork_block_number(BLOCK_NUMBER - 1)
        .timeout(20000_000_u64)
        .spawn();
    let wallet: LocalWallet = anvil.keys()[0].clone().into();
    let wallet = wallet.with_chain_id(CHAIN_ID);
    let wallet_addr = wallet.address();
    let parity_provider = Provider::<Http>::connect(HTTP_RPC_URL).await;
    let http_provider = Provider::<Http>::connect(&anvil.endpoint()).await;
    let wss_provider = Provider::<Ws>::connect(&anvil.ws_endpoint()).await.unwrap();
    let parity_client = Arc::new(parity_provider);
    let anvil_client = Arc::new(http_provider.clone());
    let singer = Arc::new(SignerMiddleware::new(http_provider, wallet.clone()));
    let contract = IArbitrage::deploy(Arc::new(Arc::clone(&singer)), ())
        .unwrap()
        .send()
        .await
        .unwrap()
        .address();

    let collector = EchoCollector::new(Arc::new(wss_provider), TX_HASH.parse::<TxHash>().unwrap());
    let mut strategy = FrontrunStrategy::new(
        Arc::clone(&parity_client),
        wallet_addr,
        Some(contract),
        None,
        false,
    );
    let executor = MultiMempoolExecutor::new(Arc::clone(&anvil_client));
    let balance_before = anvil_client.get_balance(contract, None).await.unwrap();
    let mut event_stream = collector.get_event_stream().await.unwrap();
    while let Some(event) = event_stream.next().await {
        executor
            .execute(strategy.process_event(event).await.unwrap())
            .await
            .unwrap_or_default();
    }
    let balance_after = anvil_client.get_balance(contract, None).await.unwrap();
    assert!(balance_after > balance_before);
}
