use alloy_sol_types::{sol, SolCall, SolValue};
use crate::contract_utils::eth_utils::Caller;
use alloy_primitives::{FixedBytes, U256};
use kinode_process_lib::{
    eth::{Address as EthAddress},
    println,
};
use mcstructs::TeamName;
use crate::CURRENT_CHAIN_ID;

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
    pub fn release_funds(&self, winning_team: TeamName) -> anyhow::Result<FixedBytes<32>> {
        // winning team - 0 for Team1, 1 for Team2
        let winning_team: u8 = match winning_team {
            TeamName::Team1 => 0,
            TeamName::Team2 => 1,
        };
        let call = GAMELORD::releaseFundsCall { winningTeam: winning_team }.abi_encode();
        match self.caller.send_tx(
            call,
            &self.contract_address,
            1500000,
            10000000000,
            300000000,
            U256::from(0),
            *CURRENT_CHAIN_ID,
        ) {
            Ok(tx_hash) => Ok(tx_hash),
            Err(e) => Err(anyhow::anyhow!("Error setting number: {:?}", e)),
        }
    }

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

