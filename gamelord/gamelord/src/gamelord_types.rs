//use kinode_process_lib::Address;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
#[derive(Serialize, Deserialize, Debug, Clone)]
// Note that the name might need to be changed
pub struct Player {
    pub kinode_id: String,
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

pub struct ActivePlayer {
    pub kinode_id: String,
    pub minecraft_player_name: String,
    pub current_cube: Cube,
}

impl ActivePlayer {
    pub fn kinode_id(&self) -> &String {
        &self.kinode_id
    }

    pub fn minecraft_player_name(&self) -> &String {
        &self.minecraft_player_name
    }

    pub fn current_cube(&self) -> &Cube {
        &self.current_cube
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub struct Cube {
    pub center: (i32, i32, i32),
    pub side_length: i32,
}

impl Cube {
    pub fn center(&self) -> &(i32, i32, i32) {
        &self.center
    }

    pub fn side_length(&self) -> i32 {
        self.side_length
    }

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
    pub owner: String,
}

impl Region {
    pub fn cubes(&self) -> &Vec<Cube> {
        &self.cubes
    }

    pub fn owner(&self) -> &String {
        &self.owner
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct World {
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
