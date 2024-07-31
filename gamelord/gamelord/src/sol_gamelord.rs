use crate::sol_gamelord::Gamelord::PlayerInfo;
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
    kinode, println,
};
use mcstructs::TeamName;
use serde::{Deserialize, Serialize};
use std::str::FromStr;


#[derive(Serialize, Deserialize, Clone)]
pub enum Action {
    EncryptWallet {private_key: Option<String>, password: String}, // if none, will use decrypted wallet key
    DecryptWallet(String),
    SetContractAddress(String),
    GetPlayerInfo(EthAddress)
}
pub struct Caller {
    contract_address: String,
    provider: Provider,
    chain_id: u64,
    wallet: PrivateKeySigner,
}
/* ABI import */
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug)]
    Gamelord,
    "abi/Gamelord.json"
);

fn send_tx(
    provider: &Provider,
    signer: &PrivateKeySigner,
    contract_address: &str,
    call: Bytes,
    gas_limit: u128,
    max_fee_per_gas: u128,
    max_priority_fee_per_gas: u128,
    value: U256,
) -> anyhow::Result<FixedBytes<32>> {
    // get nonce
    let mut nonce = 0;
    let tx_count = provider.get_transaction_count(signer.address(), None);
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
        chain_id: 31337,
        nonce: nonce,
        to: TxKind::Call(to),
        gas_limit: gas_limit,
        max_fee_per_gas: max_fee_per_gas,
        max_priority_fee_per_gas: max_priority_fee_per_gas,
        input: call,
        value: value,
        ..Default::default()
    };

    let sig = signer.sign_transaction_sync(&mut tx)?;
    let signed = TxEnvelope::from(tx.into_signed(sig));
    let mut buf = vec![];
    signed.encode_2718(&mut buf);

    let result = provider.send_raw_transaction(buf.into());
    match result {
        Ok(tx_hash) => Ok(tx_hash),
        Err(e) => Err(anyhow::anyhow!("Error sending transaction: {:?}", e)),
    }
}

impl Caller {
    pub fn new(
        contract_address: &str,
        provider: Provider,
        chain_id: u64,
        wallet_addr: &str,
    ) -> Option<Self> {
        // get wallet address
        let wallet_address;
        if let Ok(wallet) = PrivateKeySigner::from_str(wallet_addr) {
            wallet_address = wallet;
        } else {
            return None;
        }
        Some(Self {
            contract_address: contract_address.to_string(),
            provider,
            chain_id,
            wallet: wallet_address,
        })
    }

    // pub fn increment(&self) -> anyhow::Result<FixedBytes<32>> {
    //     let call = Counter::incrementCall {}.abi_encode();

    //     match send_tx(
    //         &self.provider,
    //         &self.wallet,
    //         &self.contract_address,
    //         call.into(),
    //         1500000,
    //         10000000000,
    //         300000000,
    //         U256::from(0),
    //     ) {
    //         Ok(tx_hash) => Ok(tx_hash),
    //         Err(e) => Err(anyhow::anyhow!("Error incrementing counter: {:?}", e)),
    //     }
    // }

    // returns eth wagered and team
    pub fn get_player_info(&self, funding_address: EthAddress) -> anyhow::Result<(U256, TeamName)> {
        let call: Vec<u8> = Gamelord::getPlayerInfoCall {
            fundingAddress: funding_address,
        }
        .abi_encode();
        let tx_req = TransactionRequest::default();
        println!("contract address: {:?}", self.contract_address);
        let to = match EthAddress::from_str(&self.contract_address) {
            Ok(to) => to,
            Err(e) => return Err(anyhow::anyhow!("Error parsing contract address: {:?}", e)),
        };
        let tx = tx_req.to(to).input(call.into());
        match self.provider.call(tx, None) {
            Ok(result) => {
                println!("result: {:?}", result);
                let player_info = PlayerInfo::abi_decode(&result, false)?;
                println!("player_info: {:?}", player_info);
                let team = if player_info.team == 0 {
                    TeamName::Team1
                } else {
                    TeamName::Team2
                };
                Ok((player_info.amountWagered, team))
            }
            Err(e) => Err(anyhow::anyhow!("Error getting player info: {:?}", e)),
        }
    }
}
