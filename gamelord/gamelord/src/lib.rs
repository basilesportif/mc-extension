use kinode_process_lib::{
    await_message, call_init, http::{self}, println, Address, Message, Response, get_blob   
};

use lazy_static::lazy_static;
use std::sync::RwLock;

mod utilities;
use utilities::valid_position;
mod gamelord_types;
use gamelord_types::{Player, ConfigurationRegion, Cube, ActivePlayer, Region};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

//Here is where we store the CURRENT world config
lazy_static! {
    static ref WORLD_CONFIG: RwLock<HashMap<String, Region>> = RwLock::new(HashMap::new());
}

lazy_static!{
    static ref CUBE_TO_OWNER: RwLock<HashMap<Cube, String>> = RwLock::new(HashMap::new());
}
// Remember to change the type key type here to Address. (maybe not, it might be a MC username)
lazy_static! {
    static ref ACTIVE_PLAYERS: RwLock<HashMap<String, ActivePlayer>> = RwLock::new(HashMap::new());
}
lazy_static! {
    static ref ALLOWED_PLAYERS: RwLock<HashMap<String, Player>> = RwLock::new(HashMap::new());
}


#[derive(Serialize, Deserialize, Debug)]
enum GamelordRequestMinecraft {
    ValidateMove { minecraft_id: String, cube: Cube },
    PlayerSpawnRequest { minecraft_id: String },
    PlayerLeaveRequest { player: Player },
}

//GamelordRequestMinecraft {ValidateMove, PlayerSpawnRequest, PlayerLeaveRequest}
//GamelordRequestUI {GenerateWorld, I assume add player but that depends on UI}


//GamelordResponseMinecraft
//GamelordResponseUI (maybe not needed)
impl GamelordRequestMinecraft {
    fn parse(bytes: &[u8]) -> Result<GamelordRequestMinecraft, serde_json::Error> {
        let json_str = String::from_utf8_lossy(bytes);
        println!("Attempting to parse JSON: {}", json_str);

        match serde_json::from_str::<GamelordRequestMinecraft>(&json_str) {
            Ok(request) => {
                println!("Successfully parsed GamelordRequest: {:?}", request);
                Ok(request)
            },
            Err(e) => {
                println!("Error parsing GamelordRequest: {:?}", e);
                println!("Error occurred at position: {}", e.column());
                if let Some(line) = json_str.lines().nth(e.line() - 1) {
                    println!("Problematic line: {}", line);
                    println!("                  {}^", " ".repeat(e.column() - 1));
                }
                Err(e)
            }
        }
    }
}

//have to figure this out, since these are responses read by mcdriver, so have to update on that side
#[derive(Serialize, Deserialize, Debug)]
enum GamelordResponseMinecraft {
    ValidateMove(bool, String),
    PlayerSpawnRequestAuthorized(bool, String, Cube),
    PlayerSpawnRequestDenied(bool ,String),
}

wit_bindgen::generate!({
    path: "target/wit",
    world: "process-v0",
});


//have everything handled here
fn handle_driver_message(message: &Message) -> anyhow::Result<()> {
    println!("handle driver message entered");
    match GamelordRequestMinecraft::parse(message.body())? {
        GamelordRequestMinecraft::ValidateMove { minecraft_id, cube } => {
            let mut active_players = ACTIVE_PLAYERS.write().expect("Failed to acquire lock");
            if active_players.contains_key(&minecraft_id) {
                println!("Player {} is active in the game.", minecraft_id);
                if let Some(active_player) = active_players.get_mut(&minecraft_id) {
                    let world_config = WORLD_CONFIG.read().expect("Failed to acquire lock");
                    let (response_message, is_valid) = valid_position(&world_config, &*active_player, &cube);
                    if !is_valid {
                        // If the position is not valid, check the cube ownership and permissions
                        let cube_to_owner = CUBE_TO_OWNER.read().unwrap();
                        if let Some(owner) = cube_to_owner.get(&cube) {
                            let region = world_config.get(owner).unwrap();
                            if region.everyone_allowed || region.authorized_players.contains(&active_player.kinode_id) {
                                // If everyone is allowed or the player is an authorized player, consider the move valid
                                active_player.current_cube = cube.clone();
                                println!("Active player {} moved to cube: {:?}", active_player.kinode_id, cube);
                                let response = serde_json::to_vec(&GamelordResponseMinecraft::ValidateMove(true, "Move allowed by owner permissions.".to_string())).unwrap();
                                Response::new()
                                    .body(response)
                                    .send()
                                    .unwrap();
                            } else {
                                // If not allowed, send a negative response
                                let response = serde_json::to_vec(&GamelordResponseMinecraft::ValidateMove(false, "Move not allowed by owner permissions.".to_string())).unwrap();
                                Response::new()
                                    .body(response)
                                    .send()
                                    .unwrap();
                            }
                        } else {
                            // If no owner found, allow the move and update the active player's current cube
                            active_player.current_cube = cube.clone();
                            println!("Active player {} moved to unclaimed cube: {:?}", active_player.kinode_id, cube);
                            let response = serde_json::to_vec(&GamelordResponseMinecraft::ValidateMove(true, "Cube unclaimed, allowed to pass.".to_string())).unwrap();
                            Response::new()
                                .body(response)
                                .send()
                                .unwrap();
                        }
                    } else {
                        // If the initial position check is valid, proceed as before
                        active_player.current_cube = cube.clone();
                        println!("Active player {} moved to cube: {:?}", minecraft_id, cube);
                        let response = serde_json::to_vec(&GamelordResponseMinecraft::ValidateMove(is_valid, response_message)).unwrap();
                        Response::new()
                            .body(response)
                            .send()
                            .unwrap();
                    }
                }
            } else {
                println!("Player {} is not active in the game.", minecraft_id);
                let response = serde_json::to_vec(&GamelordResponseMinecraft::ValidateMove(false, "Player not active in the game.".to_string())).unwrap();
                Response::new()
                    .body(response)
                    .send()
                    .unwrap();
            }
            Ok(())
        },
        // this comes from MC-Driver
        GamelordRequestMinecraft::PlayerSpawnRequest{minecraft_id} => {
            println!("Gamelord request matched");
            println!("Player spawn request received for player: {:?}", minecraft_id);
            let allowed_players = match ALLOWED_PLAYERS.read() {
                Ok(players) => players,
                Err(e) => {
                    println!("Failed to acquire read lock on ALLOWED_PLAYERS: {:?}", e);
                    return Ok(());
                }
            };
            println!("checked whether player is allowed beginning of function");
            if allowed_players.contains_key(&minecraft_id) {
                let player = allowed_players.get(&minecraft_id).expect("Player should exist");
                println!("player exists");
                // we send the cube, but we haven't specified the plugin to do anything with that info
                let spawn_cube = Cube { center: (0, 0, 0), side_length: 50 };
                println!("available cubes found");
                let active_player = ActivePlayer {
                    kinode_id: player.kinode_id().clone(),
                    minecraft_player_name: player.minecraft_player_name().clone(),
                    current_cube: spawn_cube.clone(),
                };
                println!("active player created");
                let mut active_players = ACTIVE_PLAYERS.write().expect("Failed to acquire lock");
                println!("active players inserted");
                active_players.insert(player.minecraft_player_name().clone(), active_player);
                //println!("Player {} is the owner of a region with available cubes: {:?}", player.kinode_id(), available_cubes);
                let response = serde_json::to_vec(&GamelordResponseMinecraft::PlayerSpawnRequestAuthorized(true, "Player added.".to_string(), spawn_cube.clone())).unwrap();
                Response::new()
                    .body(response)
                    .send()
                    .unwrap();
                
            } else {
                let response = serde_json::to_vec(&GamelordResponseMinecraft::PlayerSpawnRequestDenied(false, "Player not added.".to_string())).unwrap();
                Response::new()
                    .body(response)
                    .send()
                    .unwrap();
                
                println!("Player {} is not allowed on the server", minecraft_id);
                Response::new()
                    .body(b"Player not added.")
                    .send()
                    .unwrap();
            }
            Ok(())
        },
        // Think about whether I need this
        GamelordRequestMinecraft::PlayerLeaveRequest{player} => {
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
fn handle_http_ui_request(message: &Message) -> anyhow::Result<()> {
    let our_http_request = serde_json::from_slice::<http::HttpServerRequest>(message.body()).unwrap();
    match our_http_request {
        http::HttpServerRequest::Http(http_request) => {
            match http_request.method().unwrap() {
                http::Method::GET => {
                    if let Ok(path) = http_request.path() {
                        match path.as_str() {
                            "/world_config" => {
                                let world_config = WORLD_CONFIG.read().unwrap();
                                let response = serde_json::to_string(&*world_config).unwrap();
                                http::send_response(http::StatusCode::OK, None, response.into_bytes());
                            },
                            "/active_players" => {
                                let active_players = ACTIVE_PLAYERS.read().unwrap();
                                let response = serde_json::to_string(&*active_players).unwrap();
                                http::send_response(http::StatusCode::OK, None, response.into_bytes());
                            },
                            _ => http::send_response(http::StatusCode::NOT_FOUND, None, b"Not Found".to_vec()),
                        }
                    } else {
                        http::send_response(http::StatusCode::INTERNAL_SERVER_ERROR, None, b"Internal Server Error".to_vec());
                    }
                },
                http::Method::POST => {
                    if let Ok(path) = http_request.path() {
                        match path.as_str() {
                            "/api/loadWorld" => {
                                // Directly access the body (assuming it's already fully available)
                                let body = get_blob().unwrap_or_default();
                                println!("body: {:?}", body);
                                let body_str = String::from_utf8_lossy(&body.bytes);
                                println!("body_str: {:?}", body_str); // This should be the raw bytes of the body
                                match serde_json::from_str::<Vec<ConfigurationRegion>>(&body_str) {
                                    Ok(regions) => {
                                        let mut world_config = WORLD_CONFIG.write().unwrap();
                                        let mut cube_to_owner = CUBE_TO_OWNER.write().unwrap();
                                        world_config.clear();
                                        cube_to_owner.clear();

                                        // Process each ConfigurationRegion
                                        for region in regions {
                                            let mut cubes_transformed = HashMap::new();
                                            for cube in &region.cubes {
                                                let cube_id = cube.identifier(); 
                                                cubes_transformed.insert(cube_id.clone(), cube.clone());
                                                cube_to_owner.insert(cube.clone(), region.owner.clone());
                                            }

                                            let new_region = Region {
                                                cubes: cubes_transformed,
                                                owner: region.owner.clone(),
                                                everyone_allowed: region.everyone_allowed,
                                                authorized_players: region.authorized_players.clone(),
                                            };
                                            world_config.insert(region.owner.clone(), new_region);
                                        }

                                        println!("World loaded from request");
                                        http::send_response(http::StatusCode::OK, None, b"World Loaded".to_vec());
                                    },
                                    Err(e) => {
                                        println!("Failed to parse world data: {:?}", e);
                                        http::send_response(http::StatusCode::BAD_REQUEST, None, b"Invalid world data".to_vec());
                                    }
                                }
                            },
                            "/api/addPlayer" => {
                                let body = get_blob().unwrap_or_default();
                                println!("body: {:?}", body);
                                let body_str = String::from_utf8_lossy(&body.bytes);
                                println!("body_str: {:?}", body_str);
                                match serde_json::from_str::<Player>(&body_str) {
                                    Ok(player) => {
                                        println!("player: {:?}", player);
                                        let player_clone = player.clone(); // Clone player before insertion
                                        let mut allowed_players = ALLOWED_PLAYERS.write().unwrap();
                                        allowed_players.insert(player.minecraft_player_name().clone(), player);
                                        println!("Player {} added to allowed players", player_clone.minecraft_player_name());
                                        http::send_response(http::StatusCode::OK, None, b"Player Added".to_vec());
                                    },
                                    Err(e) => {
                                        println!("Failed to parse player data: {:?}", e);
                                        http::send_response(http::StatusCode::BAD_REQUEST, None, b"Invalid player data".to_vec());
                                    }
                                }
                            },
                            "/api/deleteWorld" => {
                                let mut world_config = WORLD_CONFIG.write().unwrap();
                                let mut cube_to_owner = CUBE_TO_OWNER.write().unwrap();
                                world_config.clear();
                                cube_to_owner.clear();
                                println!("World deleted from request");
                                http::send_response(http::StatusCode::OK, None, b"World Deleted".to_vec());
                            },
                            _ => http::send_response(http::StatusCode::NOT_FOUND, None, b"Not Found".to_vec()),
                        }
                    } else {
                        http::send_response(http::StatusCode::INTERNAL_SERVER_ERROR, None, b"Internal Server Error".to_vec());
                    }
                },
                _ => http::send_response(http::StatusCode::METHOD_NOT_ALLOWED, None, b"Method Not Allowed".to_vec()),
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
    // Should update this so the requests are better handled
    if is_http_request(&message) { // Check if it's an HTTP request
        println!("HTTP request received");
        handle_http_ui_request(&message)?; // Dedicated function to handle HTTP requests
    } else if message.is_local(&message.source()) {
        println!("Local message received from: {:?}", message.source());
        handle_driver_message(&message)?;
    } else{
        println!("Message from invalid source: {:?}", message.source());
    }
    Ok(())
}

call_init!(init);
fn init(our: Address) {
    println!("{our}: started");

    for path in ["/api/loadWorld", "/world_config", "/api/addPlayer", "/api/deleteWorld"] {
        http::bind_http_path(path, true, false).expect("failed to bind http path");
    }
    http::bind_http_path("/active_players", false, false).expect("failed to bind http path");
    http::serve_index_html(&our, "ui", true, false, vec!["/"]).unwrap();

    loop {
        match handle_message() {
            Ok(()) => {}
            Err(e) => {
                println!("error from somewhere: {:?}", e);
            }
        };
    }
    
}

