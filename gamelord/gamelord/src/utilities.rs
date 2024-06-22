use crate::gamelord_types::{Cube, Player};
use std::collections::HashMap;

/// Function that takes in Regions, player, and current coordinates, and returns a boolean whether a player is allowed to be there or not 
pub fn valid_position(layout: &HashMap<String, HashMap<u64, Cube>>, player: &Player, cube: &Cube) -> (String, bool) {
    let cube_id = cube.identifier();
    if let Some(cubes) = layout.get(&player.kinode_id) {
        
        if cubes.contains_key(&cube_id) {
            return (format!("Access granted to player: {} in region owned by: {}", player.kinode_id, player.kinode_id), true);
        } else {
            return (format!("Access denied to player: {}. Cube not found in your regions.", player.kinode_id), false);
        }
    } else {
        return ("Invalid player ID: No corresponding owner found.".to_string(), false);
    }
}