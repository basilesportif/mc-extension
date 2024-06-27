use kinode_process_lib::{
    await_message, call_init, http::{self}, println, Address, Message, Response
};

use lazy_static::lazy_static;
use std::sync::RwLock;

mod utilities;
use utilities::valid_position;
mod gamelord_types;
use gamelord_types::{Player, World, Cube, ActivePlayer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

//Here is where we store the CURRENT world config
lazy_static! {
    static ref WORLD_CONFIG: RwLock<HashMap<String, HashMap<u64, Cube>>> = RwLock::new(HashMap::new());
}

lazy_static! {
    static ref WORLD_SHARABLE_CONFIG: RwLock<World> = RwLock::new(World { regions: Vec::new() });
}
// Remember to change the type key type here to Address.
lazy_static! {
    static ref ACTIVE_PLAYERS: RwLock<HashMap<String, ActivePlayer>> = RwLock::new(HashMap::new());
}


#[derive(Serialize, Deserialize, Debug)]
enum GamelordRequest {
    ValidateMove(Player, Cube),
    PlayerSpawnRequest(Player),
    PlayerLeaveRequest(Player),
    GenerateWorld(World),
    DeleteWorld
}
impl GamelordRequest {
    fn parse(bytes: &[u8]) -> Result<GamelordRequest, serde_json::Error> {
        serde_json::from_slice::<GamelordRequest>(bytes)
    }
}
// The boolean might not be needed
#[derive(Serialize, Deserialize, Debug)]
enum GamelordResponse {
    ValidateMove(bool, String),
    AddPlayer(bool),
    RemovePlayer(bool),
    WorldGenerated,
    WorldDeleted,
}

wit_bindgen::generate!({
    path: "target/wit",
    world: "process-v0",
});


//have everything handled here
fn handle_kinode_message(message: &Message) -> anyhow::Result<()> {
    match GamelordRequest::parse(message.body())? {
        GamelordRequest::GenerateWorld(regions) => {
            let mut world_config = WORLD_CONFIG.write().unwrap();
            world_config.clear();
            
            for region in &regions.regions {
                let owner_cubes = world_config.entry(region.owner().clone()).or_insert_with(HashMap::new);
                for cube in region.cubes() {
                    owner_cubes.insert(cube.identifier(), cube.clone());
                }
            }
            let mut sharable_config = WORLD_SHARABLE_CONFIG.write().unwrap();
            *sharable_config = regions.clone();            

            println!("World generated with regions: {:?}", regions);
            Response::new()
            .body(serde_json::to_vec(&GamelordResponse::WorldGenerated)?)
            .send()
            .unwrap();
            Ok(())
        },
        GamelordRequest::DeleteWorld => {
            let mut world_config = WORLD_CONFIG.write().unwrap();
            world_config.clear();
            let mut sharable_config = WORLD_SHARABLE_CONFIG.write().unwrap();
            sharable_config.regions.clear();

            println!("World deleted");
            Response::new()
            .body(serde_json::to_vec(&GamelordResponse::WorldDeleted)?)
            .send()
            .unwrap();
            Ok(())
        },
        GamelordRequest::ValidateMove(player, cube) => {
            let world_config = WORLD_CONFIG.read().unwrap();
            let (response_message, is_valid) = valid_position(&world_config, &player, &cube);
            let response = serde_json::to_vec(&GamelordResponse::ValidateMove(is_valid, response_message)).unwrap();
            Response::new()
            .body(response)
            .send()
            .unwrap();
            Ok(())
        },
        GamelordRequest::PlayerSpawnRequest(player) => {
            let mut active_players = ACTIVE_PLAYERS.write().unwrap();
            if !active_players.contains_key(player.kinode_id()) {
                // to spawn a player, we first want to see whether the player can spawn (if there are any available regions)
                //active_players.insert(player.kinode_id().clone(), player);
                //To do - send response
            } else {
                println!("Player with kinode_id {} already exists.", player.kinode_id());
                //To do - send response
            }
            Ok(())
        },
        GamelordRequest::PlayerLeaveRequest(player) => {
            let mut active_players = ACTIVE_PLAYERS.write().unwrap();
            if active_players.contains_key(player.kinode_id()) {
                active_players.remove(player.kinode_id());
                println!("Player with kinode_id {} has left the game.", player.kinode_id());
            } else {
                println!("Player with kinode_id {} is not in the active players list.", player.kinode_id());
            }
            Ok(())
        },
        
    }
}

fn is_http_request(message: &Message) -> bool {
    match serde_json::from_slice::<http::HttpServerRequest>(message.body()) {
        Ok(http::HttpServerRequest::Http { .. }) => true,
        _ => false,
    }
}
fn handle_http_request(message: &Message) -> anyhow::Result<()> {
    let our_http_request = serde_json::from_slice::<http::HttpServerRequest>(message.body()).unwrap();
    match our_http_request {
        http::HttpServerRequest::Http(http_request) => {
            match http_request.method().unwrap() {
                http::Method::GET => match http_request.path() {
                    Ok(path) => {
                        match path.as_str() {
                            "/world_config" => {
                                let read_guard = WORLD_SHARABLE_CONFIG.read().unwrap();
                                let sharable_world_config: &World = &*read_guard;
                                let serialized_world_config = serde_json::to_string(sharable_world_config).expect("error serializing");
                                http::send_response(
                                    http::StatusCode::OK,
                                    None,
                                    serialized_world_config.into_bytes(),
                                );
                            }
                            _ => {
                                println!("Error handling path");
                                http::send_response(
                                    http::StatusCode::NOT_FOUND,
                                    None,
                                    b"Not Found".to_vec(),
                                );
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error retrieving path: {:?}", e);
                        http::send_response(
                            http::StatusCode::INTERNAL_SERVER_ERROR,
                            None,
                            b"Internal Server Error".to_vec(),
                        );
                    }
                },
                _ => {
                    http::send_response(
                        http::StatusCode::METHOD_NOT_ALLOWED,
                        None,
                        b"Method Not Allowed".to_vec(),
                    );
                }
            }
        }
        _ => {
            // Handle other cases or errors
        }
    }
    Ok(())
}


fn handle_message() -> anyhow::Result<()> {
    let message = await_message()?;
    println!(
        "handle_message: {:?}",
        String::from_utf8_lossy(message.body())
    );

    if is_http_request(&message) { // Check if it's an HTTP request
        println!("HTTP request received");
        handle_http_request(&message)?; // Dedicated function to handle HTTP requests
    } else if message.is_local(&message.source()) {
        println!("Local message received from: {:?}", message.source());
        handle_kinode_message(&message)?;
    } else{
        println!("Message from invalid source: {:?}", message.source());
    }
    Ok(())
}

call_init!(init);
fn init(our: Address) {
    println!("{our}: started");

    http::bind_http_path("/world_config", false, false).expect("failed to bind http path");

    loop {
        match handle_message() {
            Ok(()) => {}
            Err(e) => {
                println!("error: {:?}", e);
            }
        };
    }
    
}
