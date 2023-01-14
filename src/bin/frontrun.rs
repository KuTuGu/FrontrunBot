use arbitrage::utils::*;
use dotenv::dotenv;
use ethers::prelude::*;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let http_url = get_env("HTTP_RPC_URL");
    let wss_url = get_env("WSS_RPC_URL");
    let chain_id = get_env("CHAIN_ID").parse::<u16>().unwrap_or(1);
    let contract = get_env("CONTRACT").parse::<Address>().unwrap();
    let private_key = get_env("PRIVATE_KEY").replace("0x", "");

    let provider = Provider::<Http>::connect(&http_url).await;
    let wallet = private_key
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(chain_id);
    let flashbot = FlashBotUtil::init(provider, wallet).unwrap();

    let arbitrage = ArbitrageUtil::init(&flashbot, contract);
    let simulate = Simulate::init(&flashbot, Some(arbitrage.address()));
    let listen_poll = ListenPool::init(&wss_url, Some(1)).await;

    listen_poll
        .run(|tx_hash| {
            let simulate = &simulate;
            let flashbot = &flashbot;
            let arbitrage = &arbitrage;
            return async move {
                let tx_hash = tx_hash.clone();
                if let Ok(Some(tx_list_queue)) = simulate.run(tx_hash, false).await {
                    log(flashbot, arbitrage.address(), tx_hash, || async {
                        for tx_list in tx_list_queue {
                            // Without priority fee, all simulations will fail
                            if let Ok(tx) = arbitrage.to_tx(tx_list, true, None).await {
                                match flashbot.run(vec![tx]).await {
                                    Ok(hash) => {
                                        println!("Transaction hash: {hash:#?}");
                                    }
                                    Err(_) => {}
                                };
                            }
                        }
                    })
                    .await;
                };
            };
        })
        .await;
}
