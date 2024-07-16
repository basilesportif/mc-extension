use alloy_primitives::hex::HEX_CHARS_LOWER;
use kinode_process_lib::http::{bind_ws_path, send_ws_push, WsMessageType};
use kinode_process_lib::{
    await_message, call_init, get_blob,
    http::{self},
    println, Address, LazyLoadBlob, Message, Request, Response,
};
mod utilities;
use utilities::valid_position;
mod gamelord_types;
use gamelord_types::{ActivePlayer, Cube, CubeEffectList, TeamNameToRegion, State};
use mcstructs::{GameLobbyDiff, McClientToGamelordRequest, Player, TeamName};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Deserialize, Debug)]
enum GamelordRequestMinecraft {
    CubeTransitionRequest { minecraft_id: String, cube: Cube },
    PlayerSpawnRequest { minecraft_id: String },
    PlayerLeaveRequest { minecraft_id: String },
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
    TransitionTriggeredResponse(CubeEffectList),
    TransitionSilentResponse,
    PlayerSpawnRequestAuthorized(bool, String, Cube),
    PlayerSpawnRequestDenied(bool ,String),
}

wit_bindgen::generate!({
    path: "target/wit",
    world: "process-v0",
});

fn handle_mcclient_request(
    state: &mut State,
    ws_channel_id: &mut Option<u32>,
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
            let diff = &GameLobbyDiff::AddPlayerToTeam {
                player: player.clone(),
                team: join_team.team_name.clone(),
            };
            state.lobby.apply_diff(diff);
            state.save();
            let blob = LazyLoadBlob {
                mime: Some("application/json".to_string()),
                bytes: serde_json::to_vec(diff)?,
            };
            send_ws_push(ws_channel_id.unwrap_or(0), WsMessageType::Text, blob);

            return state
                .update_clients(&GameLobbyDiff::AddPlayerToTeam {
                    player: player.clone(),
                    team: join_team.team_name.clone(),
                });
        }
    }
}

//have everything handled here
fn handle_kinode_message(
    state: &mut State,
    ws_channel_id: &mut Option<u32>,
    message: &Message,
) -> anyhow::Result<()> {
    println!("handle kinode message entered");
    if let Ok(request) = serde_json::from_slice::<McClientToGamelordRequest>(&message.body()) {
        println!("Received request: {:?}", request);
        return handle_mcclient_request(state, ws_channel_id, &request, message);
    }
    match GamelordRequestMinecraft::parse(message.body())? {
        GamelordRequestMinecraft::CubeTransitionRequest { minecraft_id, cube } => {
            // we get information about what team the player is in based on Active players
            if let Some(active_player) = state.active_players.get_mut(&minecraft_id) {
                println!("Player {} is active in the game on team {:?}.", minecraft_id, active_player.team);
                
                // Update the player's current cube
                active_player.current_cube = cube.clone();
                
                let world_config = &state.world_config;
                let cube_to_owner = &state.cube_to_owner;

                if let Some(owners) = cube_to_owner.get(&cube) {
                    if !owners.contains(&active_player.team) {
                        // Cube is owned by enemy team(s)
                        for owner in owners {
                            if let Some(team_cubes) = world_config.get(owner) {
                                if let Some(cube_effects) = team_cubes.cubes.get(&cube) {
                                    println!("Cube effects for enemy owner {:?}: {:?}", owner, cube_effects);
                                    // Send cube effects to mcdriver
                                    let response = serde_json::to_vec(&GamelordResponseMinecraft::TransitionTriggeredResponse(
                                        cube_effects.clone()
                                    ))
                                    .expect("failed to parse gamelord cube transition response");
                                    Response::new().body(response).send().unwrap();
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
                // If we reach here, either the cube is not owned, or it's owned by the player's team
                println!("Cube not in enemy region, you are clear");
                let response = serde_json::to_vec(&GamelordResponseMinecraft::TransitionSilentResponse)
                    .expect("failed to parse gamelord cube transition response");
                Response::new().body(response).send().unwrap();
                Ok(())
            } else {
                println!("Player {} is not active in the game.", minecraft_id);
                Err(anyhow::anyhow!("Player not found in active players"))
            }
        }
        
        // this comes from MC-Driver 
        // TO DO, connect this to the team registration interface for checking
        GamelordRequestMinecraft::PlayerSpawnRequest{minecraft_id} => {
            println!("Player spawn request received for player: {:?}", minecraft_id);
            
            let team_name = if state.lobby.team1.players.iter().any(|p| p.minecraft_player_name == minecraft_id) {
                TeamName::Team1
            } else if state.lobby.team2.players.iter().any(|p| p.minecraft_player_name == minecraft_id) {
                TeamName::Team2
            } else {
                println!("Player {} is not assigned to a team", minecraft_id);
                let response = serde_json::to_vec(&GamelordResponseMinecraft::PlayerSpawnRequestDenied(
                    false,
                    "Player is not assigned to a team.".to_string(),
                )).expect("Failed to serialize response");
                Response::new().body(response).send().unwrap();
                return Ok(());
            };

            // Hardcoded spawn cube
            let spawn_cube = Cube {
                center: (0, 0, 0),
                side_length: 50,
            };

            let active_player = ActivePlayer {
                kinode_id: minecraft_id.clone(),
                minecraft_player_name: minecraft_id.clone(),
                current_cube: spawn_cube.clone(),
                team: team_name.clone(),
            };

            state.active_players.insert(minecraft_id.clone(), active_player);

            println!("Player {} added to active players on team {:?}", minecraft_id, &team_name);

            let response = serde_json::to_vec(&GamelordResponseMinecraft::PlayerSpawnRequestAuthorized(
                true,
                format!("Player added to team {:?}.", &team_name),
                spawn_cube,
            )).expect("Failed to serialize response");
            Response::new().body(response).send().unwrap();
            Ok(())
        },
        // Think about whether I need this
        GamelordRequestMinecraft::PlayerLeaveRequest{minecraft_id} => {
            if state.active_players.contains_key(&minecraft_id) {
                state.active_players.remove(&minecraft_id);
                println!(
                    "Player with kinode_id {} has left the game.",
                    &minecraft_id
                );
                state.save();
            } else {
                println!(
                    "Player with kinode_id {} is not in the active players list.",
                    &minecraft_id
                );
            }
            Ok(())
        }
    }
}

fn is_http_request(message: &Message) -> bool {
    match serde_json::from_slice::<http::HttpServerRequest>(message.body()) {
        Ok(http::HttpServerRequest::WebSocketOpen { .. }) => true,
        Ok(http::HttpServerRequest::Http(..)) => true,
        Ok(http::HttpServerRequest::WebSocketClose { .. }) => true,
        Ok(http::HttpServerRequest::WebSocketPush { .. }) => true,
        _ => false,
    }
}
fn handle_http_request(
    state: &mut State,
    ws_channel_id: &mut Option<u32>,
    message: &Message,
) -> anyhow::Result<()> {
    let our_http_request = serde_json::from_slice::<http::HttpServerRequest>(message.body())?;
    match our_http_request {
        http::HttpServerRequest::WebSocketOpen { channel_id, .. } => {
            *ws_channel_id = Some(channel_id);
            return Ok(());
        }
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
                                match serde_json::from_str::<TeamNameToRegion>(&body_str) {
                                    Ok(new_world_config) => {
                                        state.world_config = new_world_config;
                                        state.cube_to_owner.clear();
                                        // update cube_to_owner to check who owns that cube, and if it is already owned, add that owner as well
                                        for (owner, region) in state.world_config.iter() {
                                            for cube in region.cubes.keys() {
                                                state.cube_to_owner.entry(cube.clone())
                                                    .and_modify(|owners| owners.push(owner.clone()))
                                                    .or_insert_with(|| vec![owner.clone()]);
                                            }
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
                                            .insert(player.minecraft_player_name.clone(), player);
                                        println!(
                                            "Player {} added to allowed players",
                                            player_clone.minecraft_player_name
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
                            "/api/clearTeams" => {
                                state.lobby.clear_teams();
                                state.save();
                                let blob = LazyLoadBlob {
                                    mime: Some("application/json".to_string()),
                                    bytes: serde_json::to_vec(&GameLobbyDiff::Init(state.lobby.clone()))?,
                                };
                                send_ws_push(
                                    ws_channel_id.unwrap_or(0),
                                    WsMessageType::Text,
                                    blob,
                                );

                                http::send_response(
                                    http::StatusCode::OK,
                                    None,
                                    b"Teams Cleared".to_vec(),
                                );
                            }
                            "/api/editLobby" => {
                                let bytes = get_blob()
                                    .ok_or_else(|| anyhow::anyhow!("Failed to get blob"))?
                                    .bytes;
                                println!("bruuh here");
                                let edit_lobby = serde_json::from_slice::<GameLobbyDiff>(&bytes)?;

                                println!("i think here");
                                if let GameLobbyDiff::EditLobby { name, minecraft_server_address } =
                                    edit_lobby.clone()
                                {
                                    println!("but not here");
                                    // println!("state before edit_lobby: {:#?}", state);
                                    state.lobby.apply_diff(&edit_lobby);
                                    state.save();
                                    println!("here?");

                                    let blob = LazyLoadBlob {
                                        mime: Some("application/json".to_string()),
                                        bytes: serde_json::to_vec(&edit_lobby)?,
                                    };
                                    send_ws_push(
                                        ws_channel_id.unwrap_or(0),
                                        WsMessageType::Text,
                                        blob,
                                    );
                                    println!("here2?");

                                    let _ = state.update_clients(&GameLobbyDiff::EditLobby {
                                        name: name.clone(),
                                        minecraft_server_address: minecraft_server_address.clone(),
                                    });
                                    println!("here3?");

                                    // println!("state after editlobby: {:#?}", State::fetch().unwrap());
                                    http::send_response(
                                        http::StatusCode::OK,
                                        None,
                                        b"Lobby updated.".to_vec(),
                                    );
                                } else {println!("WTF WHY HERE");}
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

fn handle_message(state: &mut State, ws_channel_id: &mut Option<u32>) -> anyhow::Result<()> {
    let message = await_message()?;
    // println!(
    //     "handle_message: {:?}",
    //     String::from_utf8_lossy(message.body())
    // );

    if is_http_request(&message) {
        // Check if it's an HTTP request
        println!("HTTP request received");
        // Dedicated function to handle HTTP requests
        handle_http_request(state, ws_channel_id, &message)?; // Dedicated function to handle HTTP requests
    } else if message.is_local(&message.source()) {
        println!("Local message received from: {:?}", message.source());
        handle_kinode_message(state, ws_channel_id, &message)?;
    } else {
        println!("Message from invalid source: {:?}", message.source());
    }
    Ok(())
}

call_init!(init);
fn init(our: Address) {
    println!("{our}: gamelord started");
    let mut ws_channel_id: Option<u32> = None;
    bind_ws_path("/", true, false).unwrap();

    let _ = http::serve_ui(&our, "ui", true, false, vec!["/"]);

    for path in [
        "/api/loadWorld",
        "/world_config",
        "/api/addPlayer",
        "/api/deleteWorld",
        "/api/editLobby",
        "api/clearTeams",
        "/lobby",
    ] {
        http::bind_http_path(path, true, false).expect("failed to bind http path");
    }
    http::bind_http_path("/active_players", false, false).expect("failed to bind http path");
    http::serve_index_html(&our, "ui", true, false, vec!["/"]).unwrap();

    let mut state = State::fetch().unwrap_or_else(|| State::new(&our));
    // println!("state on init: {:?}", state);

    loop {
        match handle_message(&mut state, &mut ws_channel_id) {
            Ok(()) => {}
            Err(e) => {
                println!("error from somewhere: {:?}", e);
            }
        };
    }
}
