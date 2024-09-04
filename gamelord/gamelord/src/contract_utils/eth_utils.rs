use alloy::{
    consensus::{SignableTransaction, TxEip1559, TxEnvelope},
    network::eip2718::Encodable2718,
    network::TxSignerSync,
    primitives::TxKind,
    rpc::types::eth::TransactionRequest,
    signers::local::PrivateKeySigner,
};
use alloy_primitives::{Bytes, FixedBytes, I256, U256};
use alloy_rlp::Encodable;
use alloy_sol_types::{sol, SolCall, SolEvent, SolValue};
use kinode_process_lib::{
    eth::{Address as EthAddress, BlockId, BlockNumberOrTag, EthError, Filter, Log, Provider},
    println,
};
use std::str::FromStr;

pub struct Caller {
    pub provider: Provider,
    pub signer: PrivateKeySigner,
}

impl Caller {
    pub fn new(chain_id: u64, wallet_addr: &str) -> Option<Self> {
        // get wallet address
        let wallet_address;
        if let Ok(wallet) = PrivateKeySigner::from_str(wallet_addr) {
            wallet_address = wallet;
        } else {
            return None;
        }
        Some(Self {
            provider: Provider::new(chain_id, 5),
            signer: wallet_address,
        })
    }

    pub fn tx_req(&self, call: Vec<u8>, contract_address: &str) -> Result<alloy_primitives::Bytes, EthError> {
        let tx_req = TransactionRequest::default();
        let to = match EthAddress::from_str(contract_address) {
            Ok(to) => to,
            Err(e) => return Err(EthError::MalformedRequest),
        };
        let tx = tx_req.to(to).input(call.into());
        self.provider.call(tx, None)
    }

    pub fn send_tx(
        &self,
        call: Vec<u8>,
        contract_address: &str,
        gas_limit: u128,
        max_fee_per_gas: u128,
        max_priority_fee_per_gas: u128,
        value: U256,
        chain_id: u64,
    ) -> anyhow::Result<FixedBytes<32>> {
        // get nonce
        let mut nonce = 0;
        let tx_count = self.provider.get_transaction_count(self.signer.address(), None);
        if let Ok(tx_count) = tx_count {
            nonce = tx_count.to::<u64>();
        } else {
            println!("tx_count: {:?}", tx_count);
            return Err(anyhow::anyhow!("Error getting transaction count"));
        }

        // get contract address
        let to;
        if let Ok(address) = EthAddress::from_str(contract_address) {
            to = address;
        } else {
            return Err(anyhow::anyhow!("Invalid contract address"));
        }

        let mut tx = TxEip1559 {
            chain_id: chain_id,
            nonce: nonce,
            to: TxKind::Call(to),
            gas_limit: gas_limit,
            max_fee_per_gas: max_fee_per_gas,
            max_priority_fee_per_gas: max_priority_fee_per_gas,
            input: call.into(),
            value: value,
            ..Default::default()
        };

        let sig = self.signer.sign_transaction_sync(&mut tx)?;
        let signed = TxEnvelope::from(tx.into_signed(sig));
        let mut buf = vec![];
        signed.encode_2718(&mut buf);

        let result = self.provider.send_raw_transaction(buf.into());
        match result {
            Ok(tx_hash) => Ok(tx_hash),
            Err(e) => Err(anyhow::anyhow!("Error sending transaction: {:?}", e)),
        }
    }
}
