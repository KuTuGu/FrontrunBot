use ethers::prelude::*;
use ethers::utils::format_units;
use std::env;
use std::future::Future;

pub fn get_env(name: &str) -> String {
    env::var(name).expect(&format!("Expect environment variable <{}>", name))
}

pub async fn log_profit<M: Middleware, S: Signer, Fut: Future<Output = ()>, F: FnOnce() -> Fut>(
    client: &SignerMiddleware<M, S>,
    address: Address,
    tx_hash: TxHash,
    profit: U256,
    handle: F,
) {
    println!("\n--------------- Simulate {tx_hash:?} ---------------");
    let balance_before = client.get_balance(address, None).await.unwrap();
    handle().await;
    let balance_after = client.get_balance(address, None).await.unwrap();

    let profit = format_units(profit, "eth").unwrap();
    let balance_diff = format_units(balance_after - balance_before, "eth").unwrap();
    let balance_before = format_units(balance_before, "eth").unwrap();
    let balance_after = format_units(balance_after, "eth").unwrap();

    println!("");
    println!("Address:         {address:?}");
    println!("Expected profit: {profit:.6} eth");
    println!("Balance before:  {balance_before:.6} eth");
    println!("Balance after:   {balance_after:.6} eth");
    println!("Balance diff:    {balance_diff:.6} eth");
    println!("----------------{:-^75}----------------\n", "");
}
