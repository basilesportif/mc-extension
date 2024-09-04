use kinode_process_lib::http::{send_ws_push, WsMessageType};
use kinode_process_lib::{
    get_blob,
    http::{self},
    println, LazyLoadBlob, Message, Address,
};
use kinode_process_lib::vfs::{VfsAction, VfsRequest, create_file, open_file, create_drive};
use kinode_process_lib::Request;
use std::path::{Path, PathBuf};
use crate::{ GameLobbyDiff, TeamName };
use crate::contract_utils::gamelord_caller::GamelordCaller;
use crate::gamelord_types::State;
use mcstructs::{TeamNameToRegion, WsPush};
use crate::gamelord_types::CubeToOwnerTrait;
use serde::Deserialize;
use serde_json::Value;
use base64;
use base64::engine::general_purpose::STANDARD;
use base64::Engine; // Add this line to import the Engine trait

fn load_world(state: &mut State) {
    let body = get_blob().unwrap_or_default();
    println!("body: {:?}", body);
    let body_str = String::from_utf8_lossy(&body.bytes);
    println!("body_str: {:?}", body_str); // This should be the raw bytes of the body
    match serde_json::from_str::<TeamNameToRegion>(&body_str) {
        Ok(new_world_config) => {
            state.lobby.world_config = new_world_config;
            let _ = state
                .cube_to_owner
                .sync_with_world_config(&state.lobby.world_config);
            state.save();

            println!("World loaded from request");
            http::send_response(http::StatusCode::OK, None, b"World Loaded".to_vec());
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

fn edit_lobby(state: &mut State, ws_channel_id: &mut Option<u32>) -> anyhow::Result<()> {
    let bytes = get_blob()
        .ok_or_else(|| anyhow::anyhow!("Failed to get blob"))?
        .bytes;
    let edit_lobby = serde_json::from_slice::<GameLobbyDiff>(&bytes)?;
    if let GameLobbyDiff::EditLobby {
        name,
        minecraft_server_address,
    } = edit_lobby.clone()
    {
        if let Err(e) = state.lobby.apply_diff(&edit_lobby) {
            return Err(anyhow::anyhow!("Failed to apply lobby diff: {}", e));
        }
        state.save();

        let blob = LazyLoadBlob {
            mime: Some("application/json".to_string()),
            bytes: serde_json::to_vec(&edit_lobby)?,
        };
        send_ws_push(ws_channel_id.unwrap_or(0), WsMessageType::Text, blob);

        let _ = state.update_clients(&GameLobbyDiff::EditLobby {
            name: name.clone(),
            minecraft_server_address: minecraft_server_address.clone(),
        });

        http::send_response(http::StatusCode::OK, None, b"Lobby updated.".to_vec());
        return Ok(());
    } else {
        http::send_response(
            http::StatusCode::BAD_REQUEST,
            None,
            b"Invalid lobby update request.".to_vec(),
        );
        return Err(anyhow::anyhow!("Invalid lobby update request."));
    }
}
#[derive(Deserialize)]
struct DisperseFundsRequest {
    winning_team: TeamName,
}

pub fn handle_http_request(
    state: &mut State,
    gamelord_caller: &mut Option<GamelordCaller>,
    ws_channel_id: &mut Option<u32>,
    message: &Message,
    our: &Address,
) -> anyhow::Result<()> {
    let our_http_request = serde_json::from_slice::<http::HttpServerRequest>(message.body())?;
    match our_http_request {
        http::HttpServerRequest::WebSocketOpen { channel_id, .. } => {
            println!("got web socket open");
            *ws_channel_id = Some(channel_id);
            send_ws_push(
                ws_channel_id.unwrap_or(0),
                WsMessageType::Text,
                LazyLoadBlob {
                    mime: Some("application/json".to_string()),
                    bytes: serde_json::to_vec(&GameLobbyDiff::Init(state.lobby.clone()))?,
                },
            );

            return Ok(());
        }
        http::HttpServerRequest::WebSocketClose { .. } => {
            *ws_channel_id = None;
            return Ok(());
        }
        http::HttpServerRequest::WebSocketPush {
            channel_id,
            message_type,
        } => {
            let Some(blob) = get_blob() else {
                return Ok(());
            };

            let ws_push = serde_json::from_slice::<WsPush>(&blob.bytes)?;
            match ws_push {
                WsPush::ConfigurePoints {
                    team1_spawn,
                    team2_spawn,
                    goal_post,
                } => {
                    let diff = GameLobbyDiff::ConfigurePoints {
                        team1_spawn,
                        team2_spawn,
                        goal_post,
                    };
                    println!("diff: {:?}", diff);
                    let _ = state.lobby.apply_diff(&diff);
                    state.save();
                    let _ = state.update_clients(&diff);
                }
                _ => {}
            }
            return Ok(());
        }
        http::HttpServerRequest::Http(http_request) => {
            let method = http_request.method();
            println!("HTTP request method: {:?}", method);
            let _resp = if let Ok(path) = http_request.path() {
                println!("HTTP request path: {:?}", path);
                match path.as_str() {
                    "/world_config" => {
                        let response = serde_json::to_string(&state.lobby.world_config).unwrap();
                        http::send_response(http::StatusCode::OK, None, response.into_bytes());
                    }
                    "/active_players" => {
                        let response = serde_json::to_string(&state.active_players).unwrap();
                        http::send_response(http::StatusCode::OK, None, response.into_bytes());
                    }
                    "/api/lockGame" => {
                        state.lobby.game_started = true;
                        state.save();
                        let diff = GameLobbyDiff::GameStarted(true);
                        let _ = state.update_clients(&diff);
                        let blob = LazyLoadBlob {
                            mime: Some("application/json".to_string()),
                            bytes: serde_json::to_vec(&diff)?,
                        };
                        send_ws_push(ws_channel_id.unwrap_or(0), WsMessageType::Text, blob);
                        http::send_response(http::StatusCode::OK, None, b"Game locked".to_vec());
                    }
                    "/api/loadWorld" => load_world(state),
                    "/api/deleteWorld" => {
                        state.lobby.world_config.clear();
                        state.cube_to_owner.clear();
                        state.save();
                        println!("World deleted from request");
                        http::send_response(http::StatusCode::OK, None, b"World Deleted".to_vec());
                    }
                    "/api/disperseFunds" => {
                        let bytes = get_blob()
                            .ok_or_else(|| anyhow::anyhow!("Failed to get blob"))?
                            .bytes;
                        // Deserialize the JSON into the DisperseFundsRequest struct
                        let request: DisperseFundsRequest = serde_json::from_slice(&bytes)?;
                        let winning_team = request.winning_team;
                        println!("Dispersing funds to winning team: {:?}", winning_team);
                        if let Some(caller) = gamelord_caller {
                            let result = caller.release_funds(winning_team);
                            println!("Funds dispersed result: {:?}", result);
                            http::send_response(http::StatusCode::OK, None, b"Funds Dispersed".to_vec());
                        } else {
                            println!("No contract caller found");
                            http::send_response(http::StatusCode::INTERNAL_SERVER_ERROR, None, b"No contract caller found".to_vec());
                        }
                        let _ = state.update_clients(&GameLobbyDiff::Init(
                            state.lobby.clone().clear_teams(), // clears the WHOLE lobby, including the teams (for dispering diffs to clients)
                        ));
                        send_ws_push(ws_channel_id.unwrap_or(0), WsMessageType::Text, LazyLoadBlob {
                            mime: Some("application/json".to_string()),
                            bytes: serde_json::to_vec(&GameLobbyDiff::Init(state.lobby.clone().clear_teams()))?,
                        });
                        state.clear_teams(); // clears the state on gamelord
                        state.save();
                    }
                    "/api/clearTeams" => {
                        // need to update clients with lobby with empty teams before actually clearing teams,
                        // because it sends update to team members
                        let _ = state.update_clients(&GameLobbyDiff::Init(
                            state.lobby.clone().clear_teams(), // clears the WHOLE lobby, including the teams (for dispering diffs to clients)
                        ));
                        state.clear_teams(); // clears the state on gamelord
                        state.save();
                        http::send_response(http::StatusCode::OK, None, b"Teams Cleared".to_vec());
                    }
                    "/api/editLobby" => edit_lobby(state, ws_channel_id).unwrap_or(()),
                    "/api/loadFolder" => {
                        if let Ok(method) = http_request.method() {
                            if method == "POST" {
                                println!("Loading folder began");
                                let body = get_blob().ok_or_else(|| anyhow::anyhow!("No data received"))?;
                                let files: Value = serde_json::from_slice(&body.bytes)?;
                                
                                // Create a new drive for the uploaded folder
                                let drive_path: String = create_drive(our.package_id(), "mctex", Some(5))?;
                                
                                if let Value::Object(files_obj) = files {
                                    for (filename, content) in files_obj {
                                        if let Value::String(content_str) = content {
                                            let file_path = Path::new(&filename);
                                            let full_path = Path::new(&drive_path).join(file_path);

                                            // Create parent directories if they don't exist
                                            if let Some(parent) = full_path.parent() {
                                                create_dir_all(&drive_path, parent)?;
                                            }
                                            // Open or create the file using VFS
                                            let file = open_file(&full_path.to_string_lossy(), true, Some(5))?;

                                            // Decode base64 content
                                            let decoded_content = STANDARD.decode(content_str)?;

                                            // Write the content to the file
                                            file.write(&decoded_content)?;

                                            println!("Saved file: {:?}", full_path);
                                        }
                                    }
                                    http::send_response(http::StatusCode::OK, None, b"Folder structure loaded".to_vec());
                                } else {
                                    http::send_response(http::StatusCode::BAD_REQUEST, None, b"Invalid data format".to_vec());
                                }
                            } else {
                                http::send_response(http::StatusCode::METHOD_NOT_ALLOWED, None, b"Method Not Allowed".to_vec());
                            }
                        } else {
                            http::send_response(http::StatusCode::INTERNAL_SERVER_ERROR, None, b"Failed to get method".to_vec());
                        }
                    },
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
            };
        }
        _ => http::send_response(
            http::StatusCode::METHOD_NOT_ALLOWED,
            None,
            b"Method Not Allowed".to_vec(),
        ),
    }
    Ok(())
}

// Helper function to create directories recursively
fn create_dir_all(drive_path: &str, path: &Path) -> anyhow::Result<()> {
    let mut current = PathBuf::from(drive_path);
    for component in path.components() {
        current.push(component);
        let dir_path = current.to_string_lossy().to_string();
        
        let request = VfsRequest {
            path: dir_path,
            action: VfsAction::CreateDirAll,
        };
        
        let _ = Request::new()
            .target(("our", "vfs", "distro", "sys"))
            .body(serde_json::to_vec(&request)?)
            .send_and_await_response(5)?;
    }
    Ok(())
}