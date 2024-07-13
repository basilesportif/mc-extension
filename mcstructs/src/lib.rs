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

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
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

impl Team {
    pub fn team_has_player(&self, player: &Player) -> bool {
        self.players.contains(player)
    }
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
impl GameLobby {
    pub fn player_in_team(&self, player: &Player) -> Option<TeamName> {
        if self.team1.team_has_player(player) {
            Some(TeamName::Team1)
        } else if self.team2.team_has_player(player) {
            Some(TeamName::Team2)
        } else {
            None
        }
    }
    // team1 shouldn't seed team 2 chat
    pub fn team1_lobby(&self) -> GameLobby {
        let mut lobby = self.clone();
        lobby.team2.chat = String::new();
        lobby
    }
    // team2 shouldn't seed team 1 chat
    pub fn team2_lobby(&self) -> GameLobby {
        let mut lobby = self.clone();
        lobby.team1.chat = String::new();
        lobby
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameLobbyDiff {
    Message(ChatMessage),
    FullMessageHistory(Vec<ChatMessage>),
    // AddPlayerToTeam(Player, TeamName),
    // RemovePlayerFromTeam(Player, TeamName),
    // UpdateName(String),
    // UpdateMinecraftServerAddress(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub id: u64,
    pub time: u64,
    pub from: Player,
    pub to: TeamName,
    pub msg: String,
}
