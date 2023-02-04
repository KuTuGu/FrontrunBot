use arbitrage::utils::*;
use dotenv::dotenv;
use ethers::{prelude::*, utils::Anvil};

#[tokio::test]
#[should_panic(expected = "the method trace_call does not exist/is not available")]
async fn t_bnb() {
    dotenv().ok();
    const HTTP_RPC_URL: &str = "https://rpc.ankr.com/bsc";
    const CHAIN_ID: u64 = 56;
    const BLOCK_NUMBER: u32 = 23844530;
    const TX_HASH: &str = "0xea108fe94bfc9a71bb3e4dee4a1b0fd47572e6ad6aba8b2155ac44861be628ae";

    let anvil = Anvil::new()
        .chain_id(CHAIN_ID)
        .port(8545_u16)
        .fork(HTTP_RPC_URL)
        .fork_block_number(BLOCK_NUMBER - 1)
        .timeout(20000_000_u64)
        .spawn();

    let wallet: LocalWallet = anvil.keys()[0].clone().into();
    let wallet = wallet.with_chain_id(CHAIN_ID);
    let anvil_provider = Provider::<Http>::connect(&anvil.endpoint()).await;
    let anvil_client = SignerMiddleware::new(anvil_provider, wallet.clone());
    let arbitrage = ArbitrageUtil::deploy(&anvil_client).await.unwrap();

    let provider = Provider::<Http>::connect(HTTP_RPC_URL).await;
    let client = SignerMiddleware::new(provider, wallet.clone());
    let simulate = Simulate::init(&client, Some(arbitrage.address()))
        .await
        .unwrap();
    let tx_hash = TX_HASH.parse::<TxHash>().unwrap();

    // bnb forked at geth, does not support trace_call
    simulate.run(tx_hash, true).await.unwrap().unwrap();
}
