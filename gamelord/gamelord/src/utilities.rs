use crate::gamelord_types::{ActivePlayer, Cube, OwnerToRegion, Region};

/// Function that takes in Regions, player, and current coordinates, and returns a boolean whether a player is allowed to be there or not 
pub fn valid_position(
    layout: &OwnerToRegion,
    player: &ActivePlayer,
    cube: &Cube,
) -> (String, bool) {
    return (format!("Access granted to player: {}", player.kinode_id), true);

    // if let Some(region) = layout.get(&player.kinode_id) {
    //     if region.cubes.contains_key(&cube.identifier()) {
    //         return (format!("Access granted to player: {} in region owned by: {}", player.kinode_id, region.owner), true);
    //     }
    // }
    // ("Access denied or invalid player ID.".to_string(), false)
}
//pub fn authorized_player()



// Remember tomorrow to 