use crate::gamelord_types::{Owner, Owner::TeamName};
use mcstructs::{Cube, GameLobby, Player, Region};
use std::collections::HashMap;
use dotenvy::from_read;
use std::env;
use lazy_static::lazy_static;
use std::io::Cursor;

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
            if region.to_hashmap().contains_key(&cube) {
                return (
                    format!("Access granted to player: {}", player.kinode_id),
                    true,
                );
            }
        }
        return ("Owner not in map.".to_string(), false);
    }
    return ("Player not in either Team.".to_string(), false);
}
//pub fn authorized_player()

pub fn get_env_str(key: &str) -> String {
    let env_content = include_str!("../../../.env");
    from_read(Cursor::new(env_content)).expect("Failed to parse .env content");
    env::var(key).expect(format!("{} must be set", key).as_str())
}