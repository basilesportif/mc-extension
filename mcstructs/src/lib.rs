use kinode_process_lib::{println, Address, NodeId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Player {
    pub kinode_id: NodeId,
    pub minecraft_player_name: String,
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
    pub messages: Vec<ChatMessage>,
    pub last_message_id: u64,
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
    Init,               // requests for all gamelobby data on initialization
    JoinTeam(JoinTeam), // request to join team
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLobby {
    pub name: String,
    pub minecraft_server_address: String,
    pub team1: Team,
    pub team2: Team,
}
impl GameLobby {
    pub fn new() -> GameLobby {
        GameLobby {
            name: "Game 1".to_string(),
            minecraft_server_address: "".to_string(),
            team1: Team {
                name: TeamName::Team1,
                players: HashSet::new(),
                messages: Vec::new(),
                last_message_id: 0,
            },
            team2: Team {
                name: TeamName::Team2,
                players: HashSet::new(),
                messages: Vec::new(),
                last_message_id: 0,
            },
        }
    }
    pub fn player_in_team(&self, player: &Player) -> Option<TeamName> {
        if self.team1.team_has_player(player) {
            Some(TeamName::Team1)
        } else if self.team2.team_has_player(player) {
            Some(TeamName::Team2)
        } else {
            None
        }
    }
    // team1 shouldn't see team 2 chat, and vice versa
    pub fn lobby_for_team(&self, team: TeamName) -> GameLobby {
        let mut lobby = self.clone();
        if team == TeamName::Team1 {
            lobby.team2.messages = Vec::new();
            lobby.team2.last_message_id = 0;
        } else {
            lobby.team1.messages = Vec::new();
            lobby.team1.last_message_id = 0;
        }
        lobby
    }
    pub fn clear_teams(&mut self) -> Self {
        self.team1 = Team {
            name: TeamName::Team1,
            players: HashSet::new(),
            messages: Vec::new(),
            last_message_id: 0,
        };
        self.team2 = Team {
            name: TeamName::Team2,
            players: HashSet::new(),
            messages: Vec::new(),
            last_message_id: 0,
        };
        self.clone()
    }
    pub fn apply_diff(&mut self, diff: &GameLobbyDiff) -> Result<GameLobby, String> {
        match diff {
            GameLobbyDiff::Init(lobby) => {
                *self = lobby.clone();
                println!("lobby after diff: {:#?}", self);
                Ok(self.clone())
            }
            GameLobbyDiff::AddPlayerToTeam { player, team } => {
                let all_players: HashSet<Player> = self
                    .team1
                    .players
                    .union(&self.team2.players)
                    .cloned()
                    .collect();
                if all_players.iter().any(|p| p.kinode_id == player.kinode_id) {
                    println!("Player {} already exists in the game", player.kinode_id);
                    return Err("Player already exists in the game".to_string());
                }
                match team {
                    TeamName::Team1 => {
                        self.team1.players.insert(player.clone());
                        Ok(self.clone())
                    }
                    TeamName::Team2 => {
                        self.team2.players.insert(player.clone());
                        Ok(self.clone())
                    }
                }
            }
            GameLobbyDiff::EditLobby { name, minecraft_server_address } => {
                self.name = name.clone();
                self.minecraft_server_address = minecraft_server_address.clone();
                // println!("lobby after diff: {:#?}", self);
                Ok(self.clone())
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameLobbyDiff {
    Init(GameLobby),
    AddPlayerToTeam { player: Player, team: TeamName },
    EditLobby { name: String, minecraft_server_address: String },
    // TODO
    // Message(ChatMessage),
    // FullMessageHistory(Vec<ChatMessage>),
    // RemovePlayerFromTeam(Player, TeamName),
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct ChatMessage {
    pub id: u64,
    pub time: u64,
    pub from: Player,
    pub to: TeamName,
    pub msg: String,
}

#[derive(Debug, Clone, Deserialize)]
pub enum ChatRequest {
    SendMessage(String),
}
