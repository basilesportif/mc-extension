use crate::gamelord_types::{Cube, Player, Regions, CurrentPosition};
use crate::println;

pub fn get_owner_of_position(regions: &Regions, position: &CurrentPosition) -> Option<String> {
    for region in &regions.regions {
        for cube in &region.cubes {
            if is_position_within_cube(cube, position) {
                return Some(region.owner.clone());
            }
        }
    }
    None
}

/// Function that takes in Regions, player, and current coordinates, and returns a boolean whether a player is allowed to be there or not 
pub fn valid_position(regions: &Regions, player: &Player, position: &CurrentPosition) -> bool {
    let candidate = get_owner_of_position(regions, position);
    match candidate{
        Some(potential_candidate) => {
            if potential_candidate == player.kinode_id {
                println!("Potential candidate: {} , KinodeID: {}", potential_candidate, player.kinode_id);
                true
            } else{
                println!("Player is not allowed in this position");
                false
            }
        },
        //In case of none, no owner indicates it is not a valid position
        None => {
            println!("Invalid region");
            false}
    }
}

pub fn is_position_within_cube(cube: &Cube, position: &CurrentPosition) -> bool {
    let dx = (position.x - cube.center.0).abs();
    let dy = (position.y - cube.center.1).abs();
    let dz = (position.z - cube.center.2).abs();
    dx <= cube.radius && dy <= cube.radius && dz <= cube.radius
}