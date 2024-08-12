use kinode_process_lib::{println, NodeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, HashMap};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};


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
    pub spawn_point: Cube,
}

impl Team {
    pub fn team_has_player(&self, player: &Player) -> bool {
        self.players.contains(player)
    }
    pub fn team_has_kinode_id(&self, kinode_id: &NodeId) -> bool {
        self.players.iter().any(|p| p.kinode_id == *kinode_id)
    }
    pub fn kinode_id_to_player(&self, kinode_id: &NodeId) -> Option<Player> {
        self.players.iter().find(|p| p.kinode_id == *kinode_id).cloned()
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
    SendMessage(String),// sends message to chat to which they belong
    WorldConfigFull(TeamNameToRegion), // overwriting everytime before we implement diffs
    WorldConfigRegion(TeamName, Region),
    ReadyPlayer(NodeId),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WsPush {
    GetInit,
    SendMessage(String),
    WorldConfigRegion(TeamName, Region),
    ConfigurePoints {
        team1_spawn: Cube,
        team2_spawn: Cube,
        goal_post: Cube,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLobby {
    pub name: String,
    pub minecraft_server_address: String,
    pub world_config: TeamNameToRegion,
    pub goal_post: Cube,
    pub team1: Team,
    pub team2: Team,
    pub game_started: bool,
    pub ready_players: HashSet<NodeId>,
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
                spawn_point: Cube::new(),
            },
            team2: Team {
                name: TeamName::Team2,
                players: HashSet::new(),
                messages: Vec::new(),
                last_message_id: 0,
                spawn_point: Cube::new(),
            },
            world_config: HashMap::new(),
            goal_post: Cube::new(),
            game_started: false,
            ready_players: HashSet::new(),
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
    pub fn kinode_id_in_team(&self, kinode_id: &NodeId) -> Option<TeamName> {
        if self.team1.team_has_kinode_id(kinode_id) {
            Some(TeamName::Team1)
        } else if self.team2.team_has_kinode_id(kinode_id) {
            Some(TeamName::Team2)
        } else {
            None
        }
    }
    pub fn kinode_id_to_player(&self, kinode_id: &NodeId) -> Option<Player> {
        if self.team1.team_has_kinode_id(kinode_id) {
            self.team1.kinode_id_to_player(kinode_id)
        } else if self.team2.team_has_kinode_id(kinode_id) {
            self.team2.kinode_id_to_player(kinode_id)
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
            spawn_point: Cube::new(),
        };
        self.team2 = Team {
            name: TeamName::Team2,
            players: HashSet::new(),
            messages: Vec::new(),
            last_message_id: 0,
            spawn_point: Cube::new(),
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
            GameLobbyDiff::Message(message) => {
                let team = self.player_in_team(&message.from);
                if let Some(team) = team {
                    match team {
                        TeamName::Team1 => {
                            self.team1.messages.push(message.clone());
                            self.team1.last_message_id = message.id;
                        }
                        TeamName::Team2 => {
                            self.team2.messages.push(message.clone());
                            self.team2.last_message_id = message.id;
                        }
                    }
                } else {
                    return Err("Player not in team".to_string());
                }
                Ok(self.clone())
            }
            GameLobbyDiff::WorldConfigFull(world_config) => {
                self.world_config = world_config.clone();
                Ok(self.clone())
            }
            GameLobbyDiff::WorldConfigRegion(team, region) => {
                self.world_config.insert(team.clone(), region.clone());
                Ok(self.clone())
            }
            GameLobbyDiff::ConfigurePoints { team1_spawn, team2_spawn, goal_post } => {
                self.team1.spawn_point = team1_spawn.clone();
                self.team2.spawn_point = team2_spawn.clone();
                self.goal_post = goal_post.clone();
                Ok(self.clone())
            }   
            GameLobbyDiff::GameStarted(game_started) => {
                self.game_started = *game_started;
                Ok(self.clone())
            }
            GameLobbyDiff::ReadyPlayer(kinode_id) => {
                self.ready_players.insert(kinode_id.clone());
                Ok(self.clone())
            }
            GameLobbyDiff::GameOver { game_started, world_config } => {
                self.game_started = game_started.clone();
                self.world_config = world_config.clone();
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
    Message(ChatMessage),
    WorldConfigFull(TeamNameToRegion),
    WorldConfigRegion(TeamName, Region),
    ConfigurePoints { team1_spawn: Cube, team2_spawn: Cube, goal_post: Cube },
    GameStarted(bool),
    ReadyPlayer(NodeId),
    GameOver {
        game_started: bool,
        world_config: TeamNameToRegion,
    },
    // WorldConfigDiff
    // RemovePlayerFromTeam(Player, TeamName),
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct ChatMessage {
    pub id: u64,
    pub time: u64,
    pub from: Player,
    pub msg: String,
}

#[derive(Debug, Clone, Deserialize)]
pub enum ChatRequest {
    SendMessage(String),
}

pub type TeamNameToRegion = HashMap<TeamName, Region>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Region {
    pub cubes: Vec<(Cube, CubeEffectList)>,
}
impl Region {
    pub fn to_hashmap(&self) -> HashMap<Cube, CubeEffectList> {
        self.cubes.iter().cloned().collect()
        }
    }


#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub struct Cube {
    pub center: (i32, i32, i32),
    pub side_length: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Effect{
    Slowness,
    Weakness,
    Nausea,
    Hunger,
    Blindness,
    Poison,
    Wither,
    Levitation,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CubeEffectList{
    effects: Vec<Effect>
}

impl Cube {
    pub fn identifier(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
    pub fn new() -> Cube {
        Cube {
            center: (0,0,0),
            side_length: 16
        }
    }
}

impl PartialEq for Cube {
    fn eq(&self, other: &Self) -> bool {
        self.center == other.center && self.side_length == other.side_length
    }
}

impl Hash for Cube {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.center.hash(state);
        self.side_length.hash(state);
    }
}