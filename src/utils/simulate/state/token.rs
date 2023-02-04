use super::base::AnalyzeState;
use crate::utils::SimulateTrace;
use async_trait::async_trait;
use ethers::prelude::*;
use std::error::Error;

// @dev Analyze whether the contract token (erc20, erc223, erc777, etc.) is profitable
// @return The profit convert to native token
pub struct AnalyzeToken;

#[async_trait]
impl<'a, M, S> AnalyzeState<'a, M, S> for AnalyzeToken {
    async fn init(client: &'a SignerMiddleware<M, S>) -> Result<Self, Box<dyn Error + 'a>> {
        Ok(Self)
    }

    async fn run(
        &self,
        tx: &Transaction,
        trace: &SimulateTrace,
    ) -> Result<Option<U256>, Box<dyn Error + 'a>> {
        Ok(None)
    }
}
