use arbitrage::utils::*;
use dotenv::dotenv;
use ethers::{prelude::*, utils::Anvil};
use std::cmp;

#[tokio::test]
async fn t_16298449() {
    dotenv().ok();
    const HTTP_RPC_URL: &str = "https://rpc.ankr.com/eth";
    const CHAIN_ID: u64 = 1;
    const BLOCK_NUMBER: u32 = 16298449;
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
    let anvil_provider = Provider::<Http>::connect(&anvil.endpoint()).await;
    let anvil_client = SignerMiddleware::new(anvil_provider, wallet.clone());
    let arbitrage = ArbitrageUtil::deploy(&anvil_client).await.unwrap();

    let provider = Provider::<Http>::connect(HTTP_RPC_URL).await;
    let client = SignerMiddleware::new(provider, wallet.clone());
    let simulate = Simulate::init(&client, Some(arbitrage.address()));
    let tx_hash = TX_HASH.parse::<TxHash>().unwrap();
    let (tx_queue, profit) = simulate.run(tx_hash, true).await.unwrap().unwrap();
    log_profit(
        &anvil_client,
        arbitrage.address(),
        tx_hash,
        profit,
        || async {
            for tx_list in tx_queue {
                // No test for flashbot, for more detail, see:
                // https://github.com/foundry-rs/foundry/issues/2089
                if let Ok(tx) = arbitrage
                    .to_tx(
                        tx_list,
                        true,
                        Some(cmp::min(U256::from(12365048376181357_u64), profit * 7 / 10)),
                    )
                    .await
                {
                    let _ = anvil_client
                        .send_transaction(tx, None)
                        .await
                        .map(|pending| async {
                            println!(
                                "Transaction receipt: {:#?}",
                                pending.await.unwrap().unwrap()
                            )
                        });
                }
            }
        },
    )
    .await;
}
