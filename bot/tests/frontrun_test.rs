use dotenv::dotenv;
use ethers::{prelude::*, utils::Anvil};
use utils::*;

abigen!(Arbitrage, "out/Arbitrage.sol/Arbitrage.json");

#[tokio::test]
async fn frontrun() {
    const CHAIN_ID: u64 = 1;
    const BLOCK_NUMBER: u32 = 16298448;
    const TX_HASH: &str = "0x12d867ee837cec251b067319e2802c15b01dc2e18b052b95fcd6657e19ff2a5e";

    dotenv().ok();
    let http_url = get_env("HTTP_RPC_URL");

    let anvil = Anvil::new()
        .chain_id(CHAIN_ID)
        .port(8545_u16)
        .fork(&http_url)
        .fork_block_number(BLOCK_NUMBER)
        .timeout(20000_000_u64)
        .spawn();
    let wallet: LocalWallet = anvil.keys()[0].clone().into();
    let anvil_provider = Provider::<Http>::connect(&anvil.endpoint()).await;
    let anvil_client = SignerMiddleware::new(anvil_provider, wallet.clone());
    let arbitrage = ArbitrageUtil::deploy(&anvil_client).await.unwrap();

    let provider = Provider::<Http>::connect(&http_url).await;
    let client = SignerMiddleware::new(provider, wallet.clone());
    let simulate = Simulate::init(&client, Some(arbitrage.address()));
    let tx_hash = TX_HASH.parse::<TxHash>().unwrap();
    if let Ok(Some(transaction_list_queue)) = simulate.run(tx_hash, true).await {
        log(&anvil_client, arbitrage.address(), tx_hash, || async {
            for transaction_list in transaction_list_queue {
                // No test for flashbot, for more detail, see:
                // https://github.com/foundry-rs/foundry/issues/2089
                if let Ok(tx) = arbitrage
                    .to_tx(
                        transaction_list,
                        true,
                        Some(U256::from(12365048376181357_u64)),
                    )
                    .await
                {
                    match anvil_client.send_transaction(tx, None).await {
                        Ok(pending) => {
                            println!(
                                "Transaction receipt: {:#?}",
                                pending.await.unwrap().unwrap()
                            );
                        }
                        Err(_) => {}
                    };
                }
            }
        })
        .await;
    };
}
