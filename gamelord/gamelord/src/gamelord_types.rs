use kinode_process_lib::{get_state, println, set_state, Address, Request, NodeId, eth::{Provider, Address as EthAddress}};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use mcstructs::{ChatMessage, GameLobby, GameLobbyDiff, Player, Cube, CubeEffectList, TeamName, TeamNameToRegion};
use crate::sol_gamelord::{Caller, WALLET_KEY};
use alloy_primitives::U256;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActivePlayer {
    pub kinode_id: String,
    pub minecraft_player_name: String,
    pub current_cube: Cube,
    pub team: TeamName,
}
impl ActivePlayer {
    pub fn to_player(&self) -> Player {
        Player {
            kinode_id: self.kinode_id.clone(),
            minecraft_player_name: self.minecraft_player_name.clone(),
        }
    }
}

pub type CubeToOwner = HashMap<Cube, Vec<TeamName>>;
pub trait CubeToOwnerTrait {
    fn sync_with_world_config(&mut self, world_config: &TeamNameToRegion) -> Result<(), anyhow::Error>;
}

impl CubeToOwnerTrait for CubeToOwner {
    // update cube_to_owner to check who owns that cube, and if it is already owned, add that owner as well
    fn sync_with_world_config(&mut self, world_config: &TeamNameToRegion) -> Result<(), anyhow::Error> {
        self.clear();
        for (owner, region) in world_config.iter() {
            for cube in region.to_hashmap().keys() {
                self.entry(cube.clone())
                    .and_modify(|owners| owners.push(owner.clone()))
                    .or_insert_with(|| vec![owner.clone()]);
            }
        }
        Ok(())
    }
}

// TODO, change this to Team (Team1 or Team2), without the Unclaimed struct
#[derive(Serialize, Deserialize, Debug, Clone, Eq, Hash, PartialEq)]
pub enum Owner {
    TeamName(TeamName),
    Unclaimed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub our: Address,
    pub lobby: GameLobby,
    pub cube_to_owner: CubeToOwner,
    pub active_players: HashMap<String, ActivePlayer>, // Remember to change the type key type here to Address. (maybe not, it might be a MC username)
    pub allowed_players: HashMap<String, Player>,
    pub node_to_eth: HashMap<NodeId, (EthAddress, U256)>, // nodeId to EthAddress and Eth Wagered amount
}

impl State {
    pub fn new(our: &Address) -> Self {
        State {
            our: our.clone(),
            lobby: GameLobby::new(),
            cube_to_owner: HashMap::new(),
            active_players: HashMap::new(),
            allowed_players: HashMap::new(),
            node_to_eth: HashMap::new(),
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
    pub fn clear_teams(&mut self) {
        self.cube_to_owner.clear();
        self.active_players.clear();
        self.allowed_players.clear();
        self.node_to_eth.clear();
        self.lobby.clear_teams();
    }
    pub fn update_clients(&self, diff: &GameLobbyDiff) -> Result<(), anyhow::Error> {
        fn update_players(
            players: HashSet<Player>,
            diff: &GameLobbyDiff,
        ) -> Result<(), anyhow::Error> {
            for client in players.iter() {
                Request::new()
                    .body(serde_json::to_vec(diff)?)
                    .target(Address::new(
                        &client.kinode_id,
                        ("mcclient", "mcclient", "basilesex.os"),
                    ))
                    .send()?;
            }
            Ok(())
        }

        match diff {
            GameLobbyDiff::Message(ChatMessage { from, .. }) => {
                let team = self.lobby.player_in_team(&from);
                match team {
                    Some(TeamName::Team1) => {
                        let _ = update_players(self.lobby.team1.players.clone(), diff);
                    }
                    Some(TeamName::Team2) => {
                        let _ = update_players(self.lobby.team2.players.clone(), diff);
                    }
                    None => {}
                }
            }
            _ => {
                let _ = update_players(self.lobby.team1.players.clone(), diff);
                let _ = update_players(self.lobby.team2.players.clone(), diff);
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GamelordRequestMinecraft {
    CubeTransitionRequest { minecraft_id: String, cube: Cube },
    PlayerSpawnRequest { minecraft_id: String },
    PlayerLeaveRequest { minecraft_id: String },
}

impl GamelordRequestMinecraft {
    pub fn parse(bytes: &[u8]) -> Result<GamelordRequestMinecraft, serde_json::Error> {
        let json_str = String::from_utf8_lossy(bytes);
        println!("Attempting to parse JSON: {}", json_str);

        match serde_json::from_str::<GamelordRequestMinecraft>(&json_str) {
            Ok(request) => {
                println!("Successfully parsed GamelordRequest: {:?}", request);
                Ok(request)
            }
            Err(e) => {
                println!("Error parsing GamelordRequest: {:?}", e);
                println!("Error occurred at position: {}", e.column());
                if let Some(line) = json_str.lines().nth(e.line() - 1) {
                    println!("Problematic line: {}", line);
                    println!("                  {}^", " ".repeat(e.column() - 1));
                }
                Err(e)
            }
        }
    }
}

//have to figure this out, since these are responses read by mcdriver, so have to update on that side
#[derive(Serialize, Deserialize, Debug)]
pub enum GamelordResponseMinecraft {
    TransitionTriggeredResponse(CubeEffectList),
    TransitionSilentResponse,
    PlayerSpawnRequestAuthorized(bool, String, Cube),
    PlayerSpawnRequestDenied(bool ,String),
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
