use super::base::AnalyzeState;
use crate::strategies::frontrun::SimulateTrace;
use anyhow::Result;
use async_trait::async_trait;
use ethers::prelude::*;
use std::sync::Arc;

// @dev Analyze whether the contract token (erc20, erc223, erc777, etc.) is profitable
// @return The profit convert to native token
pub struct AnalyzeToken;

#[async_trait]
impl AnalyzeState for AnalyzeToken {
    fn new(_client: Arc<Provider<Http>>) -> Self {
        Self
    }

    async fn run(&self, _tx: &Transaction, _trace: &SimulateTrace) -> Result<Option<U256>> {
        Ok(None)
    }
}
