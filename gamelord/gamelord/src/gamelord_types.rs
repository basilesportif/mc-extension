use kinode_process_lib::Address;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    kinode_id: Address,
    minecraft_player_name: String,
}

pub struct Cube {
    center: (i32, i32, i32),
    radius: i32,
}

pub struct Region {
    cubes: Vec<Cube>,
    owner: Address,
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
