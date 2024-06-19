use crate::gamelord_types::{Cube, Player, Region};
use std::collections::HashMap;

/// Function that takes in Regions, player, and current coordinates, and returns a boolean whether a player is allowed to be there or not 
pub fn valid_position(layout: &HashMap<Cube, Region>, player: &Player, cube: &Cube) -> (String, bool) {
    if let Some(region) = layout.get(cube) {
        if region.owner == player.kinode_id {
            (format!("Access granted to player: {} in region owned by: {}", player.kinode_id, region.owner), true)
        } else {
            (format!("Access denied to player: {}. Region owned by: {}", player.kinode_id, region.owner), false)
        }
    } else {
        ("Invalid cube: No corresponding region found.".to_string(), false)
    }
}