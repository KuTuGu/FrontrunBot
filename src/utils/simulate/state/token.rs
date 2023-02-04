use super::base::AnalyzeState;
use crate::utils::SimulateTrace;
use ethers::prelude::*;
use std::error::Error;

// @dev Analyze whether the contract token (erc20, erc223, erc777, etc.) is profitable
// @return The profit convert to native token
pub struct AnalyzeToken;

impl<'a, M, S> AnalyzeState<'a, M, S> for AnalyzeToken {
    fn init(client: &'a SignerMiddleware<M, S>) -> Result<Self, Box<dyn Error + 'a>> {
        Ok(Self)
    }

    fn run(&self, tx: &Transaction, trace: &SimulateTrace) -> Option<U256> {
        None
    }
}
