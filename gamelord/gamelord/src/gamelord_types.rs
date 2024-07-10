use alloy_consensus::Sealed;
use kinode_process_lib::{get_state, set_state, Address, NodeId};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
// Note that the name might need to be changed
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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActivePlayer {
    pub kinode_id: String,
    pub minecraft_player_name: String,
    pub current_cube: Cube,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub struct Cube {
    pub center: (i32, i32, i32),
    pub side_length: i32,
}

impl Cube {
    pub fn identifier(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Region {
    pub cubes: HashMap<u64, Cube>,
    pub owner: Owner,
    pub everyone_allowed: bool,
    pub authorized_players: Vec<String>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigurationRegion {
    pub cubes: Vec<Cube>,
    pub owner: Owner,
    pub everyone_allowed: bool,
    pub authorized_players: Vec<String>,
}

pub type OwnerToRegion = HashMap<Owner, Region>;
pub type CubeToOwner = HashMap<Cube, Owner>;

// TODO, change this to Team (Team1 or Team2), without the Unclaimed struct
#[derive(Serialize, Deserialize, Debug, Clone, Eq, Hash, PartialEq)]
pub enum Owner {
    Unclaimed,
    Team1,
    Team2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Team1 {
    pub players: HashSet<Player>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Team2 {
    pub players: HashSet<Player>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum McClientToGamelordRequest {
    JoinTeam(JoinTeam),
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JoinTeam {
    pub gamelord_id: NodeId,
    pub minecraft_id: String,
    pub team_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditLobby {
    pub name: String,
    pub minecraft_server_address: String,
    pub clear_teams: bool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLobby {
    pub name: String,
    pub minecraft_server_address: String,
    pub team1: Team1,
    pub team2: Team2,
    pub chat: String, // TODO
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub our: Address,
    pub lobby: GameLobby,
}
impl State {
    pub fn new(our: &Address) -> Self {
        State {
            our: our.clone(),
            lobby: GameLobby {
                name: "Game 1".to_string(),
                minecraft_server_address: "".to_string(),
                team1: Team1 {
                    players: HashSet::new(),
                },
                team2: Team2 {
                    players: HashSet::new(),
                },
                chat: "".to_string(),
            },
        }
    }
    pub fn fetch() -> Option<State> {
        if let Some(state_bytes) = get_state() {
            bincode::deserialize(&state_bytes).ok()
        } else {
            None
        }
    }
    pub fn save(&self) {
        let serialized_state = bincode::serialize(self).expect("Failed to serialize state");
        set_state(&serialized_state);
    }
    pub fn reset_teams(&mut self) -> Self {
        self.lobby.team1 = Team1 {
            players: HashSet::new(),
        };
        self.lobby.team2 = Team2 {
            players: HashSet::new(),
        };
        self.clone()
    }
}

/*
    data:
- players in the game
  kinode id, minecraft id
- world ownership
- world size:
  x y z in all directions?
  cubes are represented by their centers

  region is a collection of cubes

  on load: process all the regions, and make a map of cube -> region
*/
