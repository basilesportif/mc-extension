use kinode_process_lib::{Address, NodeId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    pub kinode_id: NodeId,
    pub minecraft_player_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TeamName {
    Team1,
    Team2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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


