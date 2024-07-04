use crate::gamelord_types::{Cube, Player, Region};
use std::collections::HashMap;

/// Function that takes in Regions, player, and current coordinates, and returns a boolean whether a player is allowed to be there or not 
pub fn valid_position(
    layout: &HashMap<String, Region>,
    player: &Player,
    cube: &Cube,
) -> (String, bool) {
    if let Some(region) = layout.get(player.kinode_id()) {
        if region.cubes.contains_key(&cube.identifier()) {
            if region.everyone_allowed || region.authorized_players.contains(&player.kinode_id()) {
                return (format!("Access granted to player: {} in region owned by: {}", player.kinode_id(), region.owner), true);
            }
        }
    }
    ("Access denied or invalid player ID.".to_string(), false)
}
//pub fn authorized_player()



// Remember tomorrow to 