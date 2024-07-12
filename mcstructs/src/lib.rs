use kinode_process_lib::{Address, NodeId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Player {
    pub kinode_id: NodeId,
    pub minecraft_player_name: String,
}
impl Player {
    pub fn kinode_id(&self) -> &String {
        &self.kinode_id
    }

    pub fn minecraft_player_name(&self) -> &String {
        &self.minecraft_player_name
    }
}


#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum TeamName {
    Team1,
    Team2,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Team {
    pub name: TeamName,
    pub players: HashSet<Player>,
    pub chat: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JoinTeam {
    pub gamelord_id: NodeId,
    pub minecraft_id: String,
    pub team_name: TeamName,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum McClientToGamelordRequest {
    JoinTeam(JoinTeam),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLobby {
    pub name: String,
    pub minecraft_server_address: String,
    pub team1: Team,
    pub team2: Team,
}
