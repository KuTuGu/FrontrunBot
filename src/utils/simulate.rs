mod state;
mod strategy;

use ethers::prelude::*;
use futures::future::join_all;
use state::{base::AnalyzeState, eth::AnalyzeEth, token::AnalyzeToken};
use std::collections::HashMap;
use std::error::Error;
use std::iter::Sum;
use std::ops::Deref;

struct SumU256(U256);
impl Sum for SumU256 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self(U256::zero()), |a, b| Self(a.0 + b.0))
    }
}

pub type SimulateTrace = BlockTrace;

pub struct Simulate<'a, M, S> {
    inner: &'a SignerMiddleware<M, S>,
    contract: Option<Address>,
    state_analysis: Vec<Box<dyn AnalyzeState<'a, M, S>>>,
}

impl<'a, M, S> Deref for Simulate<'a, M, S> {
    type Target = &'a SignerMiddleware<M, S>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, M: Middleware + 'a, S: Signer + 'a> Simulate<'a, M, S> {
    // can use contract as a middleware to check balance, if not increase then revert
    pub async fn init(
        client: &'a SignerMiddleware<M, S>,
        contract: Option<Address>,
    ) -> Result<Simulate<'a, M, S>, Box<dyn Error + 'a>> {
        Ok(Self {
            inner: client,
            contract,
            state_analysis: vec![
                Box::new(AnalyzeEth::init(client).await?),
                Box::new(AnalyzeToken::init(client).await?),
            ],
        })
    }

    pub async fn run(
        &self,
        tx_hash: TxHash,
        rewind: bool,
    ) -> Result<Option<(Vec<Vec<TransactionRequest>>, U256)>, Box<dyn Error + 'a>> {
        if let Some(tx) = self.get_transaction(tx_hash).await? {
            let block: Option<BlockNumber> = match tx.block_number {
                Some(block_number) if rewind => Some((block_number - 1).into()),
                Some(block_number) if !rewind => Some(block_number.into()),
                _ => None,
            };
            if let Some((trace, profit)) = self.is_valuable(tx, block).await? {
                let tx_queue = self.to_tx_queue(&trace);
                if tx_queue.len() > 0 {
                    return Ok(Some((tx_queue, profit)));
                }
            };
        }

        Ok(None)
    }

    // Analyze whether tx is valuable according to different strategies
    // Support customize and optimize pruning for different scene.
    async fn is_valuable(
        &self,
        tx: Transaction,
        block: Option<BlockNumber>,
    ) -> Result<Option<(SimulateTrace, U256)>, Box<dyn Error + 'a>> {
        // e.g., prune for native token transfer.
        if strategy::transfer::run(&tx) {
            // e.g., for flashloan, loan first to ensure sufficient tokens.
            if strategy::flashloan::run(&tx) {
                let trace = self.to_trace(&tx, block).await?;

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

    async fn to_trace(
        &self,
        tx: &Transaction,
        block: Option<BlockNumber>,
    ) -> Result<SimulateTrace, Box<dyn Error + 'a>> {
        // only parity node support `trace_call`, recommend `ankr` rpc. (Sometimes it fails, need to retry)
        let trace = self
            .trace_call(tx, vec![TraceType::Trace, TraceType::StateDiff], block)
            .await?;

        // only geth node support `debug_traceCall`
        // let mut opts = GethDebugTracingOptions::default();
        // opts.tracer = Some("callTracer".into());
        // let trace = self
        //     .debug_trace_call(&tx, block.map(|n| BlockId::Number(n)), opts)
        //     .await?;

        Ok(trace)
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
                    from: Some(self.signer().address()),
                    to: Some(NameOrAddress::Address(data.to)),
                    data: Some(mock_tx_data(
                        &data.input,
                        data.from,
                        self.contract.unwrap_or(self.signer().address()),
                    )),
                    value: Some(data.value),
                    // Why is the gas obtained from the debug less than the original tx's gas limit?
                    gas: None,
                    // Due to EIP-1559, the minimum base fee must be sent, so please ensure that the wallet has enough gas fee.
                    // Only base fee here, change later or send priority fee to coinbase in contract to ensure that tx is packaged for priority.
                    gas_price: None,
                    nonce: None,
                });
            }
            Action::Create(data) => Some(TransactionRequest {
                chain_id: None,
                from: Some(self.signer().address()),
                to: None,
                data: Some(mock_tx_data(
                    &data.init,
                    data.from,
                    self.contract.unwrap_or(self.signer().address()),
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
