use ethers::prelude::*;

// The `flashloan` function simulate basically fails (callback interface / calldata format).
// What we expect to simulate is subtrace, so you have to prepare funds yourself firstly.
pub fn run(tx: &Transaction) -> bool {
    true
}
