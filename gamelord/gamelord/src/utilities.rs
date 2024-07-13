use crate::gamelord_types::{Cube, OwnerToRegion, Region, Owner::TeamName};
use mcstructs::{Player, GameLobby};

/// Function that takes in Regions, player, and current coordinates, and returns a boolean whether a player is allowed to be there or not 
pub fn valid_position(
    lobby: &GameLobby,
    layout: &OwnerToRegion,
    player: &Player,
    cube: &Cube,
) -> (String, bool) {
    if let Some(team) = lobby.player_in_team(player) {
        if let Some(region) = layout.get(&TeamName(team.name.clone())) {
            if region.cubes.contains_key(&cube.identifier()) {
                return (format!("Access granted to player: {} in region owned by: {:?}", player.kinode_id, region.owner), true);
            }
        }
        return ("Owner not in map.".to_string(), false); 
    } 
    return ("Player not in either Team.".to_string(), false);
}
//pub fn authorized_player()



// Remember tomorrow to 