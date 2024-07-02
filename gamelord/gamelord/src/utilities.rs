use crate::gamelord_types::{Cube, Player, CubePermissions};
use std::collections::HashMap;

/// Function that takes in Regions, player, and current coordinates, and returns a boolean whether a player is allowed to be there or not 
pub fn valid_position(
    layout: &HashMap<String, HashMap<u64, Cube>>,
    player: &Player,
    cube: &Cube,
    cube_permissions: Option<&CubePermissions>
) -> (String, bool) {
    if let Some(permissions) = cube_permissions {
        if permissions.everyone_allowed {
            return (format!("Access granted to player: {} for cube: {} (everyone allowed)", player.kinode_id, cube.identifier()), true);
        } else if permissions.authorized_players.contains(&player.kinode_id) {
            return (format!("Access granted to player: {} for cube: {}", player.kinode_id, cube.identifier()), true);
        }
    }

    if let Some(cubes) = layout.get(&player.kinode_id) {
        if cubes.contains_key(&cube.identifier()) {
            return (format!("Access granted to player: {} in region owned by: {}", player.kinode_id, player.kinode_id), true);
        } else {
            return (format!("Access denied to player: {}. Cube not found in your regions.", player.kinode_id), false);
        }
    } else {
        return ("Invalid player ID: No corresponding owner found.".to_string(), false);
    }
}

// Remember tomorrow to 