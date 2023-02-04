use crate::utils::SimulateTrace;
use ethers::prelude::*;

// @dev Analyze whether the contract token (erc20, erc223, erc777, etc.) is profitable
// @return The profit convert to native token
pub fn run(tx: &Transaction, trace: &SimulateTrace) -> Option<U256> {
    None
}
