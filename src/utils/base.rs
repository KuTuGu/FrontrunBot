use ethers::prelude::*;
use std::env;
use std::future::Future;

pub fn get_env(name: &str) -> String {
    env::var(name).expect(&format!("Expect environment variable <{}>", name))
}

pub async fn log<M: Middleware, S: Signer, Fut: Future<Output = ()>, F: FnOnce() -> Fut>(
    client: &SignerMiddleware<M, S>,
    address: Address,
    tx_hash: TxHash,
    handle: F,
) {
    println!("\n--------------- Simulate {tx_hash:?} ---------------");
    let balance_before = client.get_balance(address, None).await.unwrap();

    handle().await;

    let balance_after = client.get_balance(address, None).await.unwrap();
    println!("\nBalance before: {:?} {:?}", address, balance_before);
    println!("Balance after:  {:?} {:?}", address, balance_after);
    println!("Balance diff:   {:?}", balance_after - balance_before);
    println!("----------------{:-^75}----------------\n", "");
}
