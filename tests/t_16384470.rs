use arbitrage::utils::*;
use dotenv::dotenv;
use ethers::{prelude::*, utils::Anvil};

#[tokio::test]
async fn t_16384470() {
    dotenv().ok();
    const HTTP_RPC_URL: &str = "https://rpc.ankr.com/eth";
    const CHAIN_ID: u64 = 1;
    const BLOCK_NUMBER: u32 = 16384470;
    const TX_HASH: &str = "0x927b784148b60d5233e57287671cdf67d38e3e69e5b6d0ecacc7c1aeaa98985b";

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

    // Unable to detect contract creation and arbitrage in the same block
    assert_eq!(simulate.run(tx_hash, true).await.unwrap(), None);
}
