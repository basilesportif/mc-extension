//use kinode_process_lib::Address;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    //note to change kinode_id to Address
    pub kinode_id: String,
    pub minecraft_player_name: String,
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
    pub cubes: Vec<Cube>,
    //note to change owner to Address
    pub owner: String,
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
