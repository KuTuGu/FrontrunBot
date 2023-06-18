mod states;
mod strategies;

use crate::executors::multi_bundle::MultiFlashbotsBundle;
use crate::executors::multi_tx::MultiSubmitTx;
use crate::utils::ArbitrageContract;
use anyhow::Result;
use artemis_core::executors::mempool_executor::{GasBidInfo, SubmitTxToMempool};
use artemis_core::types::Strategy;
use async_trait::async_trait;
use ethers::prelude::*;
use ethers::types::Transaction;
use futures::future::join_all;
use states::{base::AnalyzeState, eth::AnalyzeEth, token::AnalyzeToken};
use std::collections::HashMap;
use std::iter::Sum;
use std::ops::Deref;
use std::sync::Arc;

struct SumU256(U256);
impl Sum for SumU256 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self(U256::zero()), |a, b| Self(a.0 + b.0))
    }
}

pub type SimulateTrace = BlockTrace;

pub struct FrontrunStrategy {
    inner: Arc<Provider<Http>>,
    signer: Address,
    contract: Option<ArbitrageContract>,
    state_analysis: Vec<Box<dyn AnalyzeState>>,
    uncle_protect: bool,
    block_number: Option<BlockNumber>,
    priority_percentage: Option<u64>,
}

impl Deref for FrontrunStrategy {
    type Target = Arc<Provider<Http>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[async_trait]
impl Strategy<Transaction, MultiFlashbotsBundle> for FrontrunStrategy {
    async fn sync_state(&mut self) -> Result<()> {
        Ok(())
    }

    async fn process_event(&mut self, event: Transaction) -> Option<MultiFlashbotsBundle> {
        if let Ok(Some(tx)) = self.get_transaction(event.hash()).await {
            self.block_number = tx
                .block_number
                .map(|n| n.checked_sub(U64::one()).unwrap().into());
            if let Ok(Some((trace, profit))) = self.is_valuable(tx).await {
                let bundle_list = self.to_bundle_list(self.to_tx_queue(&trace), profit).await;
                if bundle_list.len() > 0 {
                    return Some(bundle_list);
                }
            };
        }

        None
    }
}

#[async_trait]
impl Strategy<Transaction, MultiSubmitTx> for FrontrunStrategy {
    async fn sync_state(&mut self) -> Result<()> {
        Ok(())
    }

    async fn process_event(&mut self, event: Transaction) -> Option<MultiSubmitTx> {
        if let Ok(Some(tx)) = self.get_transaction(event.hash()).await {
            self.block_number = tx
                .block_number
                .map(|n| n.checked_sub(U64::one()).unwrap().into());
            if let Ok(Some((trace, profit))) = self.is_valuable(tx).await {
                let bundle_list = self
                    .to_bundle_list(self.to_tx_queue(&trace), U256::zero())
                    .await;
                if bundle_list.len() > 0 {
                    return Some(
                        bundle_list
                            .into_iter()
                            .flatten()
                            .map(|tx| SubmitTxToMempool {
                                tx,
                                gas_bid_info: self.priority_percentage.map(|percentage| {
                                    GasBidInfo {
                                        total_profit: profit,
                                        bid_percentage: percentage,
                                    }
                                }),
                            })
                            .collect::<MultiSubmitTx>(),
                    );
                }
            };
        }

        None
    }
}

impl FrontrunStrategy {
    pub fn new(
        client: Arc<Provider<Http>>,
        signer: Address,
        contract: Option<Address>,
        priority_percentage: Option<u64>,
        uncle_protect: bool,
    ) -> Self {
        Self {
            inner: Arc::clone(&client),
            signer,
            contract: contract
                .map(|addr| ArbitrageContract::init(Arc::clone(&client), signer, addr)),
            state_analysis: vec![
                Box::new(AnalyzeEth::new(Arc::clone(&client))),
                Box::new(AnalyzeToken::new(Arc::clone(&client))),
            ],
            uncle_protect,
            block_number: None,
            priority_percentage,
        }
    }

    // Analyze whether tx is valuable according to different strategies
    // Support customize and optimize pruning for different scene.
    async fn is_valuable(&self, tx: Transaction) -> Result<Option<(SimulateTrace, U256)>> {
        // e.g., prune for native token transfer.
        if strategies::transfer::run(&tx) {
            // e.g., for flashloan, loan first to ensure sufficient tokens.
            if strategies::flashloan::run(&tx) {
                let trace = self.to_trace(&tx).await?;

                let analysis = self.state_analysis.iter().map(|a| async {
                    a.run(&tx, &trace)
                        .await
                        .ok()
                        .unwrap_or_default()
                        .unwrap_or_default()
                });
                let profit = join_all(analysis)
                    .await
                    .into_iter()
                    .map(|p| SumU256(p))
                    .sum::<SumU256>()
                    .0;

                if !profit.is_zero() {
                    return Ok(Some((trace, profit)));
                }
            }
        }

        Ok(None)
    }

    async fn to_trace(&self, tx: &Transaction) -> Result<SimulateTrace> {
        // only parity node support `trace_call`, such as `ankr`.
        let trace = self
            .trace_call(
                tx,
                vec![TraceType::Trace, TraceType::StateDiff],
                self.block_number,
            )
            .await?;

        // only geth node support `debug_traceCall`, not support yet
        // let mut opts = GethDebugTracingOptions::default();
        // opts.tracer = Some("callTracer".into());
        // let trace = self
        //     .debug_trace_call(&tx, block.map(|n| BlockId::Number(n)), opts)
        //     .await?;

        Ok(trace)
    }

    async fn to_bundle_list(
        &self,
        tx_queue: Vec<Vec<TransactionRequest>>,
        profit: U256,
    ) -> MultiFlashbotsBundle {
        let mut bundle_list: MultiFlashbotsBundle = vec![];
        for tx_list in tx_queue {
            if let Some(contract) = &self.contract {
                if let Ok(tx) = contract
                    .to_tx(
                        tx_list,
                        if self.uncle_protect {
                            self.block_number.map(|n| {
                                n.as_number()
                                    .unwrap()
                                    .checked_sub(U64::one())
                                    .unwrap()
                                    .into()
                            })
                        } else {
                            None
                        },
                        self.priority_percentage.map(|percentage| {
                            profit
                                .checked_mul(U256::from(percentage))
                                .unwrap()
                                .checked_div(U256::from(100))
                                .unwrap()
                        }),
                    )
                    .await
                {
                    bundle_list.push(vec![tx]);
                }
            } else {
                bundle_list.push(tx_list.into_iter().map(|tx| tx.into()).collect::<_>());
            }
        }
        bundle_list
    }

    fn to_tx_queue(&self, trace: &SimulateTrace) -> Vec<Vec<TransactionRequest>> {
        let mut tx_queue = Vec::new();
        if let Some(trace_list) = &trace.trace {
            let mut trace_map = HashMap::new();
            for trace in trace_list {
                let mut trace_key = 0;
                for (i, v) in trace.trace_address.iter().rev().enumerate() {
                    trace_key += v * 2_usize.pow(i.try_into().unwrap()) + 1;
                }
                trace_map.insert(trace_key, trace);
            }

            // origin call
            let origin_call = trace_map.get(&0).unwrap();
            if let Some(tx) = self.to_tx(origin_call) {
                tx_queue.push(vec![tx]);
            }
            // internal call
            let mut internal_tx_list = Vec::new();
            for i in 1..=origin_call.subtraces {
                if let Some(tx) = self.to_tx(trace_map.get(&i).unwrap()) {
                    internal_tx_list.push(tx);
                } else {
                    // Part of the trace simulation failed, can still going?
                    // break;
                }
            }
            if internal_tx_list.len() > 0 {
                tx_queue.push(internal_tx_list);
            }
        }

        tx_queue
    }

    fn to_tx(&self, trace: &TransactionTrace) -> Option<TransactionRequest> {
        match &trace.action {
            Action::Call(data) => {
                return Some(TransactionRequest {
                    chain_id: None,
                    from: Some(self.signer),
                    to: Some(NameOrAddress::Address(data.to)),
                    data: Some(mock_tx_data(
                        &data.input,
                        data.from,
                        self.contract
                            .as_ref()
                            .map_or(self.signer, |contract| contract.address()),
                    )),
                    value: Some(data.value),
                    gas: None,
                    gas_price: None,
                    nonce: None,
                });
            }
            Action::Create(data) => Some(TransactionRequest {
                chain_id: None,
                from: Some(self.signer),
                to: None,
                data: Some(mock_tx_data(
                    &data.init,
                    data.from,
                    self.contract
                        .as_ref()
                        .map_or(self.signer, |contract| contract.address()),
                )),
                value: Some(data.value),
                gas: None,
                gas_price: None,
                nonce: None,
            }),
            _ => None,
        }
    }
}

fn mock_tx_data(data: &Bytes, from: Address, to: Address) -> Bytes {
    format!("{data:x}")
        .replace(&format!("{from:x}"), &format!("{to:x}"))
        .parse::<Bytes>()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::mock_tx_data;
    use ethers::prelude::*;

    #[tokio::test]
    async fn mock_tx_data_return_origin_data() {
        let data = "0x00000001".parse::<Bytes>().unwrap();
        let parse_data = mock_tx_data(&data, Address::random(), Address::random());
        assert_eq!(data, parse_data);
    }

    #[tokio::test]
    async fn mock_tx_data_replace_with_contract_address() {
        let from = Address::random();
        let contract = Address::random();
        let origin_data = format!("0x00000001{}", &format!("{from:x}"))
            .parse::<Bytes>()
            .unwrap();
        let parse_data = mock_tx_data(&origin_data, from, contract);
        assert!(origin_data != parse_data);
        assert_eq!(
            format!("{parse_data:x}"),
            format!("0x00000001{}", &format!("{contract:x}"))
        );
    }
}
