//use kinode_process_lib::Address;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    //note to change kinode_id to Address
    pub kinode_id: String,
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
    //note to change owner to Address
    pub owner: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Regions {
    pub regions: Vec<Region>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CurrentPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
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
