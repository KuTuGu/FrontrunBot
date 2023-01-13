use ethers::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::ops::Deref;
use std::ops::Sub;

#[derive(Default)]
struct AnalyzeAccountDiff {
    pub increase_balance: bool,
    pub balance_diff: U256,
    pub invalid_nonce: bool,
}

impl AnalyzeAccountDiff {
    fn run(diff: &AccountDiff, nonce: Option<U256>) -> Self {
        let mut increase_balance = false;
        let mut balance_diff = U256::zero();

        if let Diff::Changed(ChangedType { from, to }) = diff.balance {
            increase_balance = to > from;
            balance_diff = from.abs_diff(to);
        }

        Self {
            increase_balance,
            balance_diff,
            // The difference means that the transaction is invalid, such as being included in the block, canceled by other transactions, etc.
            // The difference will also cause an exception balance diff (unclear why)
            invalid_nonce: match diff.nonce {
                Diff::Changed(ChangedType { from, to: _ }) if from != nonce.unwrap_or(from) => true,
                _ => false,
            },
        }
    }
}

pub struct Simulate<'a, M, S> {
    inner: &'a SignerMiddleware<M, S>,
    contract: Option<Address>,
}

impl<'a, M, S> Deref for Simulate<'a, M, S> {
    type Target = &'a SignerMiddleware<M, S>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, M: Middleware + 'a, S: Signer + 'a> Simulate<'a, M, S> {
    // can use contract as a middleware to check balance, if not increase then revert
    pub fn init(client: &'a SignerMiddleware<M, S>, contract: Option<Address>) -> Self {
        Self {
            inner: client,
            contract,
        }
    }

    pub async fn run(
        &self,
        tx_hash: TxHash,

        rewind: bool,
    ) -> Result<Option<Vec<Vec<TransactionRequest>>>, Box<dyn Error + 'a>> {
        if let Some(transaction) = self.get_transaction(tx_hash).await? {
            let block: Option<BlockNumber> = match transaction.block_number {
                Some(block_number) if rewind => Some(block_number.sub(1).into()),
                Some(block_number) if !rewind => Some(block_number.into()),
                _ => None,
            };
            if let Some(trace) = self.get_valuable_trace(transaction, block).await? {
                return Ok(self.trace_to_transaction(&trace));
            };
        }

        Ok(None)
    }

    async fn get_valuable_trace(
        &self,
        tx: Transaction,
        block: Option<BlockNumber>,
    ) -> Result<Option<BlockTrace>, Box<dyn Error + 'a>> {
        let trace = self
            .trace_call(&tx, vec![TraceType::Trace, TraceType::StateDiff], block)
            .await?;
        if let Some(state_diff) = &trace.state_diff {
            if let Some(account_diff) = state_diff.0.get(&tx.from) {
                let from_account_diff = AnalyzeAccountDiff::run(account_diff, Some(tx.nonce));
                if from_account_diff.increase_balance && !from_account_diff.invalid_nonce {
                    return Ok(Some(trace));
                };

                if let Some(to) = tx.to {
                    if let Some(account_diff) = state_diff.0.get(&to) {
                        let to_account_diff = AnalyzeAccountDiff::run(account_diff, None);
                        if to_account_diff.increase_balance
                            && !to_account_diff.invalid_nonce
                            && to_account_diff.balance_diff > from_account_diff.balance_diff
                        {
                            return Ok(Some(trace));
                        };
                    }
                }
            }
        }

        Ok(None)
    }

    // Strictly judge whether each trace or subtrace is executable.
    // Or you can customize and optimize pruning for different scene.
    // e.g., for native token withdraw, no consider.
    // e.g., for flashloan, loan first to ensure sufficient tokens.
    // Because the `flashloan` function simulate basically fails (callback interface / calldata format).
    // And what we expect to simulate is subtrace, so you have to prepare funds yourself firstly.
    fn trace_to_transaction(&self, trace: &BlockTrace) -> Option<Vec<Vec<TransactionRequest>>> {
        let mut transaction_list = Vec::new();
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
            if let Some(transaction) = self.parse_transaction_trace(origin_call) {
                transaction_list.push(vec![transaction]);
            }
            // internal call
            let mut internal_transaction_list = Vec::new();
            for i in 1..=origin_call.subtraces {
                if let Some(transaction) = self.parse_transaction_trace(trace_map.get(&i).unwrap())
                {
                    internal_transaction_list.push(transaction);
                } else {
                    // Part of the trace simulation failed, can still going?
                    // break;
                }
            }
            if internal_transaction_list.len() > 0 {
                transaction_list.push(internal_transaction_list);
            }
        }

        if transaction_list.len() > 0 {
            Some(transaction_list)
        } else {
            None
        }
    }

    fn parse_transaction_trace(&self, trace: &TransactionTrace) -> Option<TransactionRequest> {
        match &trace.action {
            Action::Call(data) => {
                return Some(TransactionRequest {
                    chain_id: None,
                    from: Some(self.signer().address()),
                    to: Some(NameOrAddress::Address(data.to)),
                    data: Some(replace_transaction_data(
                        &data.input,
                        data.from,
                        self.contract.unwrap_or(self.signer().address()),
                    )),
                    value: Some(data.value),
                    // Why is the gas obtained from the debug less than the original transaction's gas limit?
                    gas: None,
                    // Due to EIP-1559, the minimum base fee must be sent, so please ensure that the wallet has enough gas fee.
                    // Only base fee here, change later or send priority fee to coinbase in contract to ensure that the transaction is packaged for priority.
                    gas_price: None,
                    nonce: None,
                });
            }
            Action::Create(data) => Some(TransactionRequest {
                chain_id: None,
                from: Some(self.signer().address()),
                to: None,
                data: Some(replace_transaction_data(
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

fn replace_transaction_data(data: &Bytes, from: Address, to: Address) -> Bytes {
    format!("{data:x}")
        .replace(&format!("{from:x}"), &format!("{to:x}"))
        .parse::<Bytes>()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::replace_transaction_data;
    use ethers::prelude::*;

    #[tokio::test]
    async fn replace_transaction_data_return_origin_data() {
        let data = "0x00000001".parse::<Bytes>().unwrap();
        let parse_data = replace_transaction_data(&data, Address::random(), Address::random());
        assert_eq!(data, parse_data);
    }

    #[tokio::test]
    async fn replace_transaction_data_replace_with_contract_address() {
        let from = Address::random();
        let contract = Address::random();
        let origin_data = format!("0x00000001{}", &format!("{from:x}"))
            .parse::<Bytes>()
            .unwrap();
        let parse_data = replace_transaction_data(&origin_data, from, contract);
        assert!(origin_data != parse_data);
        assert_eq!(
            format!("{parse_data:x}"),
            format!("0x00000001{}", &format!("{contract:x}"))
        );
    }
}
