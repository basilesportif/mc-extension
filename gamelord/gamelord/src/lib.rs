use kinode_process_lib::{
    await_message, call_init, get_blob,
    http::{self},
    println, set_state, Address, Message, Response,
};

use lazy_static::lazy_static;
use std::sync::RwLock;

mod utilities;
use utilities::valid_position;
mod gamelord_types;
use gamelord_types::{
    ActivePlayer, ConfigurationRegion, Cube, CubeToOwner, EditLobby, McClientToGamelordRequest,
    OwnerToRegion, Player, Region, State,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

//Here is where we store the CURRENT world config
lazy_static! {
    static ref WORLD_CONFIG: RwLock<OwnerToRegion> = RwLock::new(HashMap::new());
}
lazy_static! {
    static ref CUBE_TO_OWNER: RwLock<CubeToOwner> = RwLock::new(HashMap::new());
}

// Remember to change the type key type here to Address. (maybe not, it might be a MC username)
lazy_static! {
    static ref ACTIVE_PLAYERS: RwLock<HashMap<String, ActivePlayer>> = RwLock::new(HashMap::new());
}
lazy_static! {
    static ref ALLOWED_PLAYERS: RwLock<HashMap<String, Player>> = RwLock::new(HashMap::new());
}

#[derive(Serialize, Deserialize, Debug)]
enum GamelordRequest {
    ValidateMove { minecraft_id: String, cube: Cube },
    PlayerSpawnRequest { minecraft_id: String },
    PlayerLeaveRequest { player: Player },
    GenerateWorld { regions: Vec<ConfigurationRegion> },
    DeleteWorld,
}
impl GamelordRequest {
    fn parse(bytes: &[u8]) -> Result<GamelordRequest, serde_json::Error> {
        let json_str = String::from_utf8_lossy(bytes);
        println!("Attempting to parse JSON: {}", json_str);

        match serde_json::from_str::<GamelordRequest>(&json_str) {
            Ok(request) => {
                println!("Successfully parsed GamelordRequest: {:?}", request);
                Ok(request)
            }
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
// The boolean might not be needed
#[derive(Serialize, Deserialize, Debug)]
enum GamelordResponse {
    ValidateMove(bool, String),
    AddPlayer(bool, String, Cube),
    AddPlayerFailed(bool, String),
    RemovePlayer(bool),
    WorldGenerated,
    WorldDeleted,
}

wit_bindgen::generate!({
    path: "target/wit",
    world: "process-v0",
});

//have everything handled here
fn handle_kinode_message(state: &mut State, message: &Message) -> anyhow::Result<()> {
    println!("handle kinode message entered");

    if let Ok(request) = serde_json::from_slice::<McClientToGamelordRequest>(&message.body()) {
        println!("Received request: {:?}", request);
        let McClientToGamelordRequest::JoinTeam(join_team) = request;
        let player = Player {
            kinode_id: message.source().node().to_string(),
            minecraft_player_name: join_team.minecraft_id.to_string(),
        };
        match join_team.team_name.as_str() {
            "Team1" => state.lobby.team1.players.insert(player),
            "Team2" => state.lobby.team2.players.insert(player),
            _ => {
                println!("Invalid team name: {}", join_team.team_name);
                return Ok(());
            }
        };
        state.save();
        println!("state after join team: {:?}", state);
        return Ok(());
    }

    match GamelordRequest::parse(message.body())? {
        GamelordRequest::GenerateWorld { regions } => {
            let regions_clone = regions.clone();
            let mut world_config = WORLD_CONFIG.write().unwrap();
            let mut cube_to_owner = CUBE_TO_OWNER.write().unwrap();
            world_config.clear();
            cube_to_owner.clear();

            for region in regions {
                // Assuming regions is a Vec<ConfigurationRegion>
                let mut cubes_transformed = HashMap::new();
                for cube in &region.cubes {
                    let cube_id = cube.identifier(); // Use the identifier method to get the key
                    cubes_transformed.insert(cube_id.clone(), cube.clone()); // Insert into the new HashMap
                    cube_to_owner.insert(cube.clone(), region.owner.clone()); // Map cube to owner
                }

                let new_region = Region {
                    cubes: cubes_transformed, // Use the transformed HashMap
                    owner: region.owner.clone(),
                    everyone_allowed: region.everyone_allowed,
                    authorized_players: region.authorized_players.clone(),
                };
                world_config.insert(region.owner.clone(), new_region);
            }
            println!("World generated with regions: {:?}", &regions_clone); // Use cloned data
            Response::new()
                .body(serde_json::to_vec(&GamelordResponse::WorldGenerated)?)
                .send()
                .unwrap();
            Ok(())
        }
        // figure out where the deletion of the world can come from
        GamelordRequest::DeleteWorld => {
            let mut world_config = WORLD_CONFIG.write().unwrap();
            world_config.clear();
            println!("World deleted");
            Response::new()
                .body(serde_json::to_vec(&GamelordResponse::WorldDeleted)?)
                .send()
                .unwrap();
            Ok(())
        }
        GamelordRequest::ValidateMove { minecraft_id, cube } => {
            let mut active_players = ACTIVE_PLAYERS.write().expect("Failed to acquire lock");
            if active_players.contains_key(&minecraft_id) {
                println!("Player {} is active in the game.", minecraft_id);
                if let Some(active_player) = active_players.get_mut(&minecraft_id) {
                    let world_config = WORLD_CONFIG.read().expect("Failed to acquire lock");
                    let (response_message, is_valid) =
                        valid_position(&world_config, &*active_player, &cube);
                    if !is_valid {
                        // If the position is not valid, check the cube ownership and permissions
                        let cube_to_owner = CUBE_TO_OWNER.read().unwrap();
                        if let Some(owner) = cube_to_owner.get(&cube) {
                            let region = world_config.get(owner).unwrap();
                            if region.everyone_allowed
                                || region.authorized_players.contains(&active_player.kinode_id)
                            {
                                // If everyone is allowed or the player is an authorized player, consider the move valid
                                active_player.current_cube = cube.clone();
                                println!(
                                    "Active player {} moved to cube: {:?}",
                                    active_player.kinode_id, cube
                                );
                                let response = serde_json::to_vec(&GamelordResponse::ValidateMove(
                                    true,
                                    "Move allowed by owner permissions.".to_string(),
                                ))
                                .unwrap();
                                Response::new().body(response).send().unwrap();
                            } else {
                                // If not allowed, send a negative response
                                let response = serde_json::to_vec(&GamelordResponse::ValidateMove(
                                    false,
                                    "Move not allowed by owner permissions.".to_string(),
                                ))
                                .unwrap();
                                Response::new().body(response).send().unwrap();
                            }
                        } else {
                            // If no owner found, allow the move and update the active player's current cube
                            active_player.current_cube = cube.clone();
                            println!(
                                "Active player {} moved to unclaimed cube: {:?}",
                                active_player.kinode_id, cube
                            );
                            let response = serde_json::to_vec(&GamelordResponse::ValidateMove(
                                true,
                                "Cube unclaimed, allowed to pass.".to_string(),
                            ))
                            .unwrap();
                            Response::new().body(response).send().unwrap();
                        }
                    } else {
                        // If the initial position check is valid, proceed as before
                        active_player.current_cube = cube.clone();
                        println!("Active player {} moved to cube: {:?}", minecraft_id, cube);
                        let response = serde_json::to_vec(&GamelordResponse::ValidateMove(
                            is_valid,
                            response_message,
                        ))
                        .unwrap();
                        Response::new().body(response).send().unwrap();
                    }
                }
            } else {
                println!("Player {} is not active in the game.", minecraft_id);
                let response = serde_json::to_vec(&GamelordResponse::ValidateMove(
                    false,
                    "Player not active in the game.".to_string(),
                ))
                .unwrap();
                Response::new().body(response).send().unwrap();
            }
            Ok(())
        }
        // this comes from MC-Driver
        GamelordRequest::PlayerSpawnRequest { minecraft_id } => {
            println!("Gamelord request matched");
            println!(
                "Player spawn request received for player: {:?}",
                minecraft_id
            );
            let allowed_players = match ALLOWED_PLAYERS.read() {
                Ok(players) => players,
                Err(e) => {
                    println!("Failed to acquire read lock on ALLOWED_PLAYERS: {:?}", e);
                    return Ok(());
                }
            };
            println!("checked whether player is allowed beginning of function");
            if allowed_players.contains_key(&minecraft_id) {
                let player = allowed_players
                    .get(&minecraft_id)
                    .expect("Player should exist");
                println!("player exists");
                //let available_cubes = world_config.get("gamelord").map_or_else(|| Vec::new(), |region| region.cubes.values().cloned().collect());
                // for now its the first one, let's set the first available cube as the players 'spawn' point
                let spawn_cube = Cube {
                    center: (0, 0, 0),
                    side_length: 50,
                };
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
                let response = serde_json::to_vec(&GamelordResponse::AddPlayer(
                    true,
                    "Player added.".to_string(),
                    spawn_cube.clone(),
                ))
                .unwrap();
                Response::new().body(response).send().unwrap();
            } else {
                let response = serde_json::to_vec(&GamelordResponse::AddPlayerFailed(
                    false,
                    "Player not added.".to_string(),
                ))
                .unwrap();
                Response::new().body(response).send().unwrap();

                println!("Player {} is not allowed on the server", minecraft_id);
                Response::new().body(b"Player not added.").send().unwrap();
            }
            Ok(())
        }
        GamelordRequest::PlayerLeaveRequest { player } => {
            let mut active_players = ACTIVE_PLAYERS.write().unwrap();
            if active_players.contains_key(player.kinode_id()) {
                active_players.remove(player.kinode_id());
                println!(
                    "Player with kinode_id {} has left the game.",
                    player.kinode_id()
                );
            } else {
                println!(
                    "Player with kinode_id {} is not in the active players list.",
                    player.kinode_id()
                );
            }
            Ok(())
        }
    }
}

fn is_http_request(message: &Message) -> bool {
    match serde_json::from_slice::<http::HttpServerRequest>(message.body()) {
        Ok(http::HttpServerRequest::Http { .. }) => true,
        _ => false,
    }
}
fn handle_http_request(state: &mut State, message: &Message) -> anyhow::Result<()> {
    let our_http_request =
        serde_json::from_slice::<http::HttpServerRequest>(message.body()).unwrap();
    match our_http_request {
        http::HttpServerRequest::Http(http_request) => {
            match http_request.method().unwrap() {
                http::Method::GET => {
                    if let Ok(path) = http_request.path() {
                        match path.as_str() {
                            "/world_config" => {
                                let world_config = WORLD_CONFIG.read().unwrap();
                                let response = serde_json::to_string(&*world_config).unwrap();
                                http::send_response(
                                    http::StatusCode::OK,
                                    None,
                                    response.into_bytes(),
                                );
                            }
                            "/active_players" => {
                                let active_players = ACTIVE_PLAYERS.read().unwrap();
                                let response = serde_json::to_string(&*active_players).unwrap();
                                http::send_response(
                                    http::StatusCode::OK,
                                    None,
                                    response.into_bytes(),
                                );
                            }
                            "/lobby" => {
                                let lobby = state.lobby.clone();
                                let response = serde_json::to_string(&lobby).unwrap();
                                http::send_response(
                                    http::StatusCode::OK,
                                    None,
                                    response.into_bytes(),
                                );
                            }
                            _ => http::send_response(
                                http::StatusCode::NOT_FOUND,
                                None,
                                b"Not Found".to_vec(),
                            ),
                        }
                    } else {
                        http::send_response(
                            http::StatusCode::INTERNAL_SERVER_ERROR,
                            None,
                            b"Internal Server Error".to_vec(),
                        );
                    }
                }
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
                                                cubes_transformed
                                                    .insert(cube_id.clone(), cube.clone());
                                                cube_to_owner
                                                    .insert(cube.clone(), region.owner.clone());
                                            }

                                            let new_region = Region {
                                                cubes: cubes_transformed,
                                                owner: region.owner.clone(),
                                                everyone_allowed: region.everyone_allowed,
                                                authorized_players: region
                                                    .authorized_players
                                                    .clone(),
                                            };
                                            world_config.insert(region.owner.clone(), new_region);
                                        }

                                        println!("World loaded from request");
                                        http::send_response(
                                            http::StatusCode::OK,
                                            None,
                                            b"World Loaded".to_vec(),
                                        );
                                    }
                                    Err(e) => {
                                        println!("Failed to parse world data: {:?}", e);
                                        http::send_response(
                                            http::StatusCode::BAD_REQUEST,
                                            None,
                                            b"Invalid world data".to_vec(),
                                        );
                                    }
                                }
                            }
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
                                        allowed_players
                                            .insert(player.minecraft_player_name().clone(), player);
                                        println!(
                                            "Player {} added to allowed players",
                                            player_clone.minecraft_player_name()
                                        );
                                        http::send_response(
                                            http::StatusCode::OK,
                                            None,
                                            b"Player Added".to_vec(),
                                        );
                                    }
                                    Err(e) => {
                                        println!("Failed to parse player data: {:?}", e);
                                        http::send_response(
                                            http::StatusCode::BAD_REQUEST,
                                            None,
                                            b"Invalid player data".to_vec(),
                                        );
                                    }
                                }
                            }
                            "/api/deleteWorld" => {
                                let mut world_config = WORLD_CONFIG.write().unwrap();
                                let mut cube_to_owner = CUBE_TO_OWNER.write().unwrap();
                                world_config.clear();
                                cube_to_owner.clear();
                                println!("World deleted from request");
                                http::send_response(
                                    http::StatusCode::OK,
                                    None,
                                    b"World Deleted".to_vec(),
                                );
                            }
                            "/api/editLobby" => {
                                let bytes = get_blob()
                                    .ok_or_else(|| anyhow::anyhow!("Failed to get blob"))?
                                    .bytes;
                                let edit_lobby = serde_json::from_slice::<EditLobby>(&bytes)?;

                                println!("state before edit_lobby: {:#?}", state);
                                state.lobby.name = edit_lobby.name;
                                state.lobby.minecraft_server_address =
                                    edit_lobby.minecraft_server_address;
                                if edit_lobby.clear_teams {
                                    state.clear_teams().save();
                                } else {
                                    state.save()
                                }
                                println!("state after editlobby: {:#?}", State::fetch().unwrap());
                                http::send_response(
                                    http::StatusCode::OK,
                                    None,
                                    b"Lobby updated.".to_vec(),
                                );
                            }
                            _ => http::send_response(
                                http::StatusCode::NOT_FOUND,
                                None,
                                b"Not Found".to_vec(),
                            ),
                        }
                    } else {
                        http::send_response(
                            http::StatusCode::INTERNAL_SERVER_ERROR,
                            None,
                            b"Internal Server Error".to_vec(),
                        );
                    }
                }
                _ => http::send_response(
                    http::StatusCode::METHOD_NOT_ALLOWED,
                    None,
                    b"Method Not Allowed".to_vec(),
                ),
            }
        }
        _ => {
            // Handle other cases or errors
        }
    }
    Ok(())
}

fn handle_message(state: &mut State) -> anyhow::Result<()> {
    let message = await_message()?;
    // println!(
    //     "handle_message: {:?}",
    //     String::from_utf8_lossy(message.body())
    // );

    if is_http_request(&message) {
        // Check if it's an HTTP request
        println!("HTTP request received");
        handle_http_request(state, &message)?; // Dedicated function to handle HTTP requests
    } else if message.is_local(&message.source()) {
        println!("Local message received from: {:?}", message.source());
        handle_kinode_message(state, &message)?;
    } else {
        println!("Message from invalid source: {:?}", message.source());
    }
    Ok(())
}

call_init!(init);
fn init(our: Address) {
    println!("{our}: started");

    for path in [
        "/api/loadWorld",
        "/world_config",
        "/api/addPlayer",
        "/api/deleteWorld",
        "/api/editLobby",
        "/lobby",
    ] {
        http::bind_http_path(path, true, false).expect("failed to bind http path");
    }
    http::bind_http_path("/active_players", false, false).expect("failed to bind http path");
    http::serve_index_html(&our, "ui", true, false, vec!["/"]).unwrap();

    let mut state = State::fetch().unwrap_or_else(|| State::new(&our));
    println!("state on init: {:?}", state);

    loop {
        match handle_message(&mut state) {
            Ok(()) => {}
            Err(e) => {
                println!("error from somewhere: {:?}", e);
            }
        };
    }
}
