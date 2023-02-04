use super::base::{AnalyzeState, DiffAnalysis};
use crate::utils::SimulateTrace;
use async_trait::async_trait;
use ethers::prelude::*;
use std::error::Error;

// Analyze whether the native token is profitable.
pub struct AnalyzeEth;

#[async_trait]
impl<'a, M, S> AnalyzeState<'a, M, S> for AnalyzeEth {
    async fn init(client: &'a SignerMiddleware<M, S>) -> Result<Self, Box<dyn Error + 'a>> {
        Ok(Self)
    }

    async fn run(
        &self,
        tx: &Transaction,
        trace: &SimulateTrace,
    ) -> Result<Option<U256>, Box<dyn Error + 'a>> {
        let mut profit = U256::zero();

        if let Some(state_diff) = &trace.state_diff {
            if let Some(account_diff) = state_diff.0.get(&tx.from) {
                let from_account_diff = DiffAnalysis::init(account_diff, Some(tx.nonce));
                if from_account_diff.increase_balance && !from_account_diff.invalid_nonce {
                    profit += from_account_diff.balance_diff;
                };

                if let Some(to) = tx.to {
                    if let Some(account_diff) = state_diff.0.get(&to) {
                        let to_account_diff = DiffAnalysis::init(account_diff, None);
                        if to_account_diff.increase_balance
                            && !to_account_diff.invalid_nonce
                            && to_account_diff.balance_diff > from_account_diff.balance_diff
                        {
                            profit += to_account_diff.balance_diff;
                        };
                    }
                }
            }
        }

        if profit.is_zero() {
            Ok(None)
        } else {
            Ok(Some(profit))
        }
    }
}
