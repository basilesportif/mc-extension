use kinode_process_lib::Address;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    pub kinode_id: Address,
    pub minecraft_player_name: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cube {
    pub center: (i32, i32, i32),
    pub radius: i32,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Region {
    pub cubes: Vec<Cube>,
    pub owner: Address,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Regions {
    pub regions: Vec<Region>,
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
