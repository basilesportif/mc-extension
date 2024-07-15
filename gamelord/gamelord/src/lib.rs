use kinode_process_lib::{
    await_message, call_init, get_blob,
    http::{self},
    println, Address, Message, Request, Response,
};
use lazy_static::lazy_static;
use std::sync::RwLock;

mod utilities;
use utilities::valid_position;
mod gamelord_types;
use gamelord_types::{ActivePlayer, ConfigurationRegion, Cube, EditLobby, Region, State};
use mcstructs::{GameLobby, GameLobbyDiff, McClientToGamelordRequest, Player, TeamName};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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

//have to figure this out, since these are responses read by mcdriver, so have to update on that side
#[derive(Serialize, Deserialize, Debug)]
enum GamelordResponseMinecraft {
    ValidateMove(bool, String),
    PlayerSpawnRequestAuthorized(bool, String, Cube),
    PlayerSpawnRequestDenied(bool, String),
}

wit_bindgen::generate!({
    path: "target/wit",
    world: "process-v0",
});

fn handle_mcclient_request(
    state: &mut State,
    request: &McClientToGamelordRequest,
    message: &Message,
) -> anyhow::Result<()> {
    match request.clone() {
        McClientToGamelordRequest::Init => {
            for player in state.lobby.team1.players.iter() {
                if player.kinode_id == message.source().node() {
                    let diff =
                        GameLobbyDiff::Init(state.lobby.clone().lobby_for_team(TeamName::Team1));
                    println!("Sending {:?} to {:?}", diff, player.kinode_id);
                    Request::new()
                        .body(serde_json::to_vec(&diff)?)
                        .target(Address::new(
                            &player.kinode_id,
                            ("mcclient", "mcclient", "basilesex.os"),
                        ))
                        .send()?;
                }
            }
            for player in state.lobby.team2.players.iter() {
                if player.kinode_id == message.source().node() {
                    let diff =
                        GameLobbyDiff::Init(state.lobby.clone().lobby_for_team(TeamName::Team1));
                    println!("Sending {:?} to {:?}", diff, player.kinode_id);
                    Request::new()
                        .body(serde_json::to_vec(&diff)?)
                        .target(Address::new(
                            &player.kinode_id,
                            ("mcclient", "mcclient", "basilesex.os"),
                        ))
                        .send()?;
                }
            }
            return Ok(());
        }
        McClientToGamelordRequest::JoinTeam(join_team) => {
            let player = Player {
                kinode_id: message.source().node().to_string(),
                minecraft_player_name: join_team.minecraft_id.to_string(),
            };

            let all_players: HashSet<Player> = state
                .lobby
                .team1
                .players
                .union(&state.lobby.team2.players)
                .cloned()
                .collect();
            if all_players.contains(&player) {
                println!("Player {} already exists in the game", player.kinode_id);
                return Ok(());
            }

            match join_team.team_name {
                TeamName::Team1 => state.lobby.team1.players.insert(player.clone()),
                TeamName::Team2 => state.lobby.team2.players.insert(player.clone()),
                _ => {
                    println!("Invalid team name: {:?}", join_team.team_name);
                    return Ok(());
                }
            };
            state.save();
            return state
                .update_clients(GameLobbyDiff::AddPlayerToTeam(player, join_team.team_name));
        }
    }
}

//have everything handled here
fn handle_kinode_message(state: &mut State, message: &Message) -> anyhow::Result<()> {
    println!("handle kinode message entered");
    if let Ok(request) = serde_json::from_slice::<McClientToGamelordRequest>(&message.body()) {
        println!("Received request: {:?}", request);
        return handle_mcclient_request(state, &request, message);
    }
    match GamelordRequestMinecraft::parse(message.body())? {
        GamelordRequestMinecraft::ValidateMove { minecraft_id, cube } => {
            if state.active_players.contains_key(&minecraft_id) {
                println!("Player {} is active in the game.", minecraft_id);
                if let Some(active_player) = state.active_players.get_mut(&minecraft_id) {
                    let (response_message, is_valid) = valid_position(
                        &state.lobby,
                        &state.world_config,
                        &active_player.to_player(),
                        &cube,
                    );
                    if !is_valid {
                        // If the position is not valid, check the cube ownership and permissions
                        if let Some(owner) = state.cube_to_owner.get(&cube) {
                            let region = state.world_config.get(owner).unwrap();
                            if region.everyone_allowed
                                || region.authorized_players.contains(&active_player.kinode_id)
                            {
                                // If everyone is allowed or the player is an authorized player, consider the move valid
                                active_player.current_cube = cube.clone();
                                println!(
                                    "Active player {} moved to cube: {:?}",
                                    active_player.kinode_id, cube
                                );
                                let response =
                                    serde_json::to_vec(&GamelordResponseMinecraft::ValidateMove(
                                        true,
                                        "Move allowed by owner permissions.".to_string(),
                                    ))
                                    .unwrap();
                                Response::new().body(response).send().unwrap();
                            } else {
                                // If not allowed, send a negative response
                                let response =
                                    serde_json::to_vec(&GamelordResponseMinecraft::ValidateMove(
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
                            let response =
                                serde_json::to_vec(&GamelordResponseMinecraft::ValidateMove(
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
                        let response = serde_json::to_vec(
                            &GamelordResponseMinecraft::ValidateMove(is_valid, response_message),
                        )
                        .unwrap();
                        Response::new().body(response).send().unwrap();
                    }
                }
            } else {
                println!("Player {} is not active in the game.", minecraft_id);
                let response = serde_json::to_vec(&GamelordResponseMinecraft::ValidateMove(
                    false,
                    "Player not active in the game.".to_string(),
                ))
                .unwrap();
                Response::new().body(response).send().unwrap();
            }
            Ok(())
        }
        // this comes from MC-Driver
        GamelordRequestMinecraft::PlayerSpawnRequest { minecraft_id } => {
            println!("Gamelord request matched");
            println!(
                "Player spawn request received for player: {:?}",
                minecraft_id
            );
            println!("checked whether player is allowed beginning of function");
            if state.allowed_players.contains_key(&minecraft_id) {
                let player = state
                    .allowed_players
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
                println!("active players inserted");
                state
                    .active_players
                    .insert(player.minecraft_player_name().clone(), active_player);
                state.save();
                //println!("Player {} is the owner of a region with available cubes: {:?}", player.kinode_id(), available_cubes);
                let response =
                    serde_json::to_vec(&GamelordResponseMinecraft::PlayerSpawnRequestAuthorized(
                        true,
                        "Player added.".to_string(),
                        spawn_cube.clone(),
                    ))
                    .unwrap();
                Response::new().body(response).send().unwrap();
            } else {
                let response =
                    serde_json::to_vec(&GamelordResponseMinecraft::PlayerSpawnRequestDenied(
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
        // Think about whether I need this
        GamelordRequestMinecraft::PlayerLeaveRequest { player } => {
            if state.active_players.contains_key(player.kinode_id()) {
                state.active_players.remove(player.kinode_id());
                println!(
                    "Player with kinode_id {} has left the game.",
                    player.kinode_id()
                );
                state.save();
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
                                let response = serde_json::to_string(&state.world_config).unwrap();
                                http::send_response(
                                    http::StatusCode::OK,
                                    None,
                                    response.into_bytes(),
                                );
                            }
                            "/active_players" => {
                                let response =
                                    serde_json::to_string(&state.active_players).unwrap();
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
                                        state.world_config.clear();
                                        state.cube_to_owner.clear();

                                        // Process each ConfigurationRegion
                                        for region in regions {
                                            let mut cubes_transformed = HashMap::new();
                                            for cube in &region.cubes {
                                                let cube_id = cube.identifier();
                                                cubes_transformed
                                                    .insert(cube_id.clone(), cube.clone());
                                                state
                                                    .cube_to_owner
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
                                            state
                                                .world_config
                                                .insert(region.owner.clone(), new_region);
                                        }

                                        state.save();

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
                                        state
                                            .allowed_players
                                            .insert(player.minecraft_player_name().clone(), player);
                                        println!(
                                            "Player {} added to allowed players",
                                            player_clone.minecraft_player_name()
                                        );
                                        state.save();
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
                                state.world_config.clear();
                                state.cube_to_owner.clear();
                                state.save();
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

                                // println!("state before edit_lobby: {:#?}", state);
                                state.lobby.name = edit_lobby.name;
                                state.lobby.minecraft_server_address =
                                    edit_lobby.minecraft_server_address;
                                if edit_lobby.clear_teams {
                                    state.lobby.clear_teams();
                                    // TODO - game ended update to everyone from teams
                                }
                                state.save();
                                // println!("state after editlobby: {:#?}", State::fetch().unwrap());
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
        // Dedicated function to handle HTTP requests
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

    let _ = http::serve_ui(&our, "ui", true, false, vec!["/"]);

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
    // println!("state on init: {:?}", state);

    loop {
        match handle_message(&mut state) {
            Ok(()) => {}
            Err(e) => {
                println!("error from somewhere: {:?}", e);
            }
        };
    }
}
