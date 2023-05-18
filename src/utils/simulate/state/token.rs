use super::base::{AnalyzeState, DiffAnalysis};
use crate::utils::{get_env, SimulateTrace};
use async_trait::async_trait;
use ethers::prelude::*;
use std::error::Error;
use std::ops::Deref;
use std::sync::Arc;

abigen!(
    UniswapV3Pool,
    r#"[
        function observe(uint32[] calldata secondsAgos) external view returns (int56[] memory tickCumulatives, uint160[] memory secondsPerLiquidityCumulativeX128s)
    ]"#,
);

// @dev Analyze whether the contract token (erc20, erc223, erc777, etc.) is profitable
// @dev trace_call doesn't trace storage changes, not support this analysis.
// @return The profit convert to native token
pub struct AnalyzeToken<'a, M, S> {
    inner: UniswapV3Pool<&'a SignerMiddleware<M, S>>,
}

impl<'a, M, S> Deref for AnalyzeToken<'a, M, S> {
    type Target = UniswapV3Pool<&'a SignerMiddleware<M, S>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[async_trait]
impl<'a, M: Middleware, S: Signer> AnalyzeState<'a, M, S> for AnalyzeToken<'a, M, S> {
    async fn init(client: &'a SignerMiddleware<M, S>) -> Result<Self, Box<dyn Error + 'a>> {
        let address = (&get_env("UNISWAP_V3_POOL")).parse::<Address>().unwrap();
        let inner = UniswapV3Pool::new(address, Arc::new(client));

        Ok(Self { inner })
    }

    async fn run(
        &self,
        tx: &Transaction,
        trace: &SimulateTrace,
    ) -> Result<Option<U256>, Box<dyn Error + 'a>> {
        let mut profit = U256::zero();
        let interval = 3600_i64; // 1h

        let (tick_cumulatives, _) = self
            .observe(vec![interval.try_into().unwrap(), 0])
            .call()
            .await?;
        let average_tick = (tick_cumulatives[0] - tick_cumulatives[1]) / interval;
        let sqrt_price_x96 = tick_math::get_sqrt_ratio_at_tick(average_tick.try_into().unwrap())?;

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

        Ok(None)
    }
}
