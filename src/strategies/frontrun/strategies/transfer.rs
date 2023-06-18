use ethers::prelude::*;

// Only filter native token transfer tx.
// Won't filter the contract token，because contracts such as erc777 may have exploitable vulnerability.
pub fn run(tx: &Transaction) -> bool {
    !tx.input.is_empty()
}

#[cfg(test)]
mod tests {
    use super::run as pass_transfer_check;
    use ethers::{
        core::rand::thread_rng,
        prelude::*,
        types::transaction::eip2718::TypedTransaction,
        utils::{
            parse_ether,
            rlp::{self, Decodable},
        },
    };
    use std::sync::Arc;

    abigen!(ERC20Token, "out/Arbitrage.sol/IERC20.json");

    #[tokio::test]
    async fn filter_transfer_tx() {
        let wallet = LocalWallet::new(&mut thread_rng());
        let tx: TypedTransaction =
            TransactionRequest::pay(Address::zero(), parse_ether(1).unwrap()).into();
        let signature = wallet.sign_transaction(&tx).await.unwrap();

        let rlp_vec = tx.rlp_signed(&signature).to_vec();
        let expected_rlp = rlp::Rlp::new(&rlp_vec);
        let tx = Transaction::decode(&expected_rlp).unwrap();

        assert!(pass_transfer_check(&tx) == false);
    }

    #[tokio::test]
    async fn not_filter_contract_transfer_tx() {
        let provider = Provider::<Http>::connect("http://localhost:8545").await;
        let token = ERC20Token::new(Address::zero(), Arc::new(provider));
        let wallet = LocalWallet::new(&mut thread_rng());
        let tx = token.transfer(Address::random(), U256::zero()).tx;
        let signature = wallet.sign_transaction(&tx).await.unwrap();

        let rlp_vec = tx.rlp_signed(&signature).to_vec();
        let expected_rlp = rlp::Rlp::new(&rlp_vec);
        let tx = Transaction::decode(&expected_rlp).unwrap();

        assert!(pass_transfer_check(&tx) == true);
    }
}
