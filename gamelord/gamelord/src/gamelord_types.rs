use alloy_consensus::Sealed;
use kinode_process_lib::{get_state, set_state, Address, NodeId};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use mcstructs::{GameLobby, Team, TeamName, Player};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActivePlayer {
    pub kinode_id: String,
    pub minecraft_player_name: String,
    pub current_cube: Cube,
}
impl ActivePlayer {
    pub fn to_player(&self) -> Player {
        Player {
            kinode_id: self.kinode_id.clone(),
            minecraft_player_name: self.minecraft_player_name.clone(),
        }
    }
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


// TODO, change this to Team (Team1 or Team2), without the Unclaimed struct
#[derive(Serialize, Deserialize, Debug, Clone, Eq, Hash, PartialEq)]
pub enum Owner {
    TeamName(TeamName),
    Unclaimed,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditLobby {
    pub name: String,
    pub minecraft_server_address: String,
    pub clear_teams: bool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub our: Address,
    pub lobby: GameLobby,
    pub world_config: HashMap<Owner, Region>,
    pub cube_to_owner: HashMap<Cube, Owner>,
    pub active_players: HashMap<String, ActivePlayer>, // Remember to change the type key type here to Address. (maybe not, it might be a MC username)
    pub allowed_players: HashMap<String, Player>,
}

impl State {
    pub fn new(our: &Address) -> Self {
        State {
            our: our.clone(),
            lobby: GameLobby::new(),
            world_config: HashMap::new(),
            cube_to_owner: HashMap::new(),
            active_players: HashMap::new(),
            allowed_players: HashMap::new(),
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
    // pub fn update_clients(&self) {
        
    // }
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
