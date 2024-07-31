use alloy_sol_types::{sol, SolCall, SolEvent, SolValue};
use crate::eth_utils::Caller;
use alloy_primitives::{Bytes, FixedBytes, I256, U256};
use kinode_process_lib::{
    eth::{Address as EthAddress, BlockId, BlockNumberOrTag, EthError, Filter, Log, Provider},
    println,
};
use alloy::{
    consensus::{SignableTransaction, TxEip1559, TxEnvelope},
    network::eip2718::Encodable2718,
    network::TxSignerSync,
    primitives::TxKind,
    rpc::types::eth::TransactionRequest,
    signers::local::PrivateKeySigner,
};
use mcstructs::TeamName;
use std::str::FromStr;

/* ABI import */
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug)]
    GAMELORD,
    "abi/Gamelord.json"
);

pub struct GamelordCaller {
    pub caller: Caller,
    pub contract_address: String,
}

impl GamelordCaller {
    // pub fn increment(&self) -> anyhow::Result<FixedBytes<32>> {
    //     let call = Counter::incrementCall {}.abi_encode();

    //     match send_tx(
    //         &self.provider,
    //         &self.wallet,
    //         &self.contract_address,
    //         call,
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
        let call: Vec<u8> = GAMELORD::getPlayerInfoCall {
            fundingAddress: funding_address,
        }
        .abi_encode();
        match self.caller.tx_req(call, &self.contract_address) {
            Ok(result) => {
                println!("result: {:?}", result);
                let player_info = GAMELORD::PlayerInfo::abi_decode(&result, false)?;
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

