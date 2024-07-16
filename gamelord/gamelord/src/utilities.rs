use crate::gamelord_types::{Cube, Owner, Region, Owner::TeamName};
use mcstructs::{Player, GameLobby};
use std::collections::HashMap;

// TODO: Change this function to determine effects that are applied, else do nothing
/// Function that takes in Regions, player, and current coordinates, and returns a boolean whether a player is allowed to be there or not 
pub fn valid_position(
    lobby: &GameLobby,
    layout: &HashMap<Owner, Region>,
    player: &Player,
    cube: &Cube,
) -> (String, bool) {
    if let Some(team_name) = lobby.player_in_team(player) {
        if let Some(region) = layout.get(&TeamName(team_name.clone())) {
            if region.cubes.contains_key(&cube) {
                return (format!("Access granted to player: {}", player.kinode_id), true);
            }
        }
        return ("Owner not in map.".to_string(), false); 
    } 
    return ("Player not in either Team.".to_string(), false);
}
//pub fn authorized_player()



// Remember tomorrow to 