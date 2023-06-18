use crate::strategies::frontrun::SimulateTrace;
use anyhow::Result;
use async_trait::async_trait;
use ethers::prelude::*;
use std::sync::Arc;

#[async_trait]
pub trait AnalyzeState: Send + Sync {
    fn new(client: Arc<Provider<Http>>) -> Self
    where
        Self: Sized;

    async fn run(&self, tx: &Transaction, trace: &SimulateTrace) -> Result<Option<U256>>;
}

#[derive(Default, Debug)]
pub struct DiffAnalysis {
    pub increase_balance: bool,
    pub balance_diff: U256,
    pub invalid_nonce: bool,
}

impl DiffAnalysis {
    pub fn new(diff: &AccountDiff, nonce: Option<U256>) -> Self {
        let mut increase_balance = false;
        let mut balance_diff = U256::zero();

        if let Diff::Changed(ChangedType { from, to }) = diff.balance {
            increase_balance = to > from;
            balance_diff = from.abs_diff(to);
        }

        Self {
            increase_balance,
            balance_diff,
            // The difference means that the tx is invalid, such as being included in the block, canceled by other txs, etc.
            // The difference will also cause an exception balance diff (unclear why)
            invalid_nonce: match diff.nonce {
                Diff::Changed(ChangedType { from, to: _ }) if from != nonce.unwrap_or(from) => true,
                _ => false,
            },
        }
    }
}
