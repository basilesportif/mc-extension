use dotenvy::from_read;
use std::env;
use std::path::Path;
use lazy_static::lazy_static;
use std::io::Cursor;

use chrono::Utc;
use kinode_process_lib::http::{bind_ws_path, send_ws_push, WsMessageType};
use kinode_process_lib::{
    await_message, call_init,
    eth::Provider,
    get_blob,
    http::{self},
    println, Address, LazyLoadBlob, Message, Request, Response,
};
mod encryption;
mod sol_gamelord;
use sol_gamelord::{Action, Caller};
mod gamelord_types;
mod utilities;
use alloy::signers::{local::PrivateKeySigner, SignerSync};
use alloy_primitives::{Signature, U256};
use alloy_signer::{LocalWallet, Signer};
use gamelord_types::{
    ActivePlayer, CubeToOwnerTrait, GamelordRequestMinecraft, GamelordResponseMinecraft,
    PrivateKey, SerializableWallet, State,
};
use mcstructs::{
    ChatMessage, Cube, CubeEffectList, GameLobbyDiff, JoinTeam, McClientToGamelordRequest, Player,
    Region, TeamName, TeamNameToRegion, WsPush,
};
use std::collections::HashMap;
use std::str::FromStr;

use crate::encryption::{decrypt_data, encrypt_data};

wit_bindgen::generate!({
    path: "target/wit",
    world: "process-v0",
});

lazy_static! {
    pub static ref CHAIN_ID: u64 = {
        let env_content = include_str!("../../../.env");
        from_read(Cursor::new(env_content)).expect("Failed to parse .env content");
        env::var("CHAIN_ID").expect("CHAIN_ID must be set").parse().unwrap()
    };

    pub static ref WETH: HashMap<u64, Address> = {
        let mut m = HashMap::new();
        m.insert(10, "0x4200000000000000000000000000000000000006".parse::<Address>().unwrap()); // Optimism
        m.insert(11155111, "0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9".parse::<Address>().unwrap()); // Sepolia
        m
    };
}


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

fn add_to_team(
    state: &mut State,
    kinode_id: String,
    minecraft_id: String,
    team_name: TeamName,
    ws_channel_id: &mut Option<u32>,
) -> anyhow::Result<()> {
    let player = Player {
        kinode_id: kinode_id,
        minecraft_player_name: minecraft_id,
    };
    let diff = &GameLobbyDiff::AddPlayerToTeam {
        player: player.clone(),
        team: team_name.clone(),
    };
    match state.lobby.apply_diff(diff) {
        Ok(lobby) => {
            state.lobby = lobby;
            state.save();
            let blob = LazyLoadBlob {
                mime: Some("application/json".to_string()),
                bytes: serde_json::to_vec(diff)?,
            };
            send_ws_push(ws_channel_id.unwrap_or(0), WsMessageType::Text, blob);

            return state.update_clients(&GameLobbyDiff::AddPlayerToTeam {
                player: player.clone(),
                team: team_name.clone(),
            });
        }
        Err(e) => {
            println!("mcclient: error applying diff: {}", e);
            return Ok(());
        }
    };
}

fn handle_mcclient_request(
    state: &mut State,
    contract_caller: &mut Option<Caller>,
    ws_channel_id: &mut Option<u32>,
    request: &McClientToGamelordRequest,
    message: &Message,
) -> anyhow::Result<()> {
    match request.clone() {
        McClientToGamelordRequest::Init => {
            // sends init only to requestor, confirms that they are in team (loops just look dumb)
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
            println!("got join team");
            let node_id = message.source().node().to_string();
            if let Some(..) = state.node_to_eth.get(&node_id) {
                println!("node already in team");
                return Ok(());
                // how to make this idempotent properly?
                // return add_to_team(
                //     state,
                //     node_id,
                //     join_team.minecraft_id,
                //     join_team.team_name,
                //     ws_channel_id,
                // );
            }
            println!("here");

            let signature = match Signature::from_str(join_team.signature.as_str()) {
                Ok(signature) => signature,
                Err(e) => return Err(anyhow::anyhow!("Error: {}", e)),
            };
            println!("here1");

            let recovered_address = signature.recover_address_from_msg(node_id.clone())?;
            if recovered_address != join_team.eth_address {
                println!("here2");

                return Err(anyhow::anyhow!("Invalid signature"));
            }
            println!("here3");

            // get eth wagered and team from chain
            let caller = match contract_caller {
                Some(caller) => {
                    println!("here4");
                    caller
                }
                None => return Ok(()),
            };

            println!("here5");

            let (amount_wagered, team) = caller.get_player_info(recovered_address)?;
            println!("here6");

            if amount_wagered < "50000000000000000".parse().unwrap() {
                return Err(anyhow::anyhow!("Player has not wagered enough ETH."));
            }

            println!("here7");

            state
                .node_to_eth
                .insert(node_id.clone(), (recovered_address, amount_wagered));
            state.save();

            println!("here8");

            let _ = add_to_team(state, node_id, join_team.minecraft_id, team, ws_channel_id);

            println!("here9");

            return Ok(());
        }
        McClientToGamelordRequest::SendMessage(chat_message) => {
            let sender_kinode_id = message.source().node().to_string();
            let sender_team = state.lobby.kinode_id_in_team(&sender_kinode_id);
            let last_msg_id = if let Some(sender_team) = sender_team {
                match sender_team {
                    TeamName::Team1 => state.lobby.team1.last_message_id,
                    TeamName::Team2 => state.lobby.team2.last_message_id,
                }
            } else {
                return Err(anyhow::anyhow!("Sender is not in a team"));
            };
            let id = last_msg_id + 1;
            let diff = GameLobbyDiff::Message({
                ChatMessage {
                    id,
                    time: Utc::now().timestamp() as u64,
                    from: state.lobby.kinode_id_to_player(&sender_kinode_id).unwrap(),
                    msg: chat_message.clone(),
                }
            });
            println!("diff: {:?}", diff);
            if let Ok(lobby) = state.lobby.apply_diff(&diff) {
                state.lobby = lobby;
                state.save();
                println!("updated clients");
                return state.update_clients(&diff);
            }
            return Ok(());
        }
        McClientToGamelordRequest::WorldConfigFull(world_config) => {
            state.lobby.world_config = world_config.clone();
            let _ = state
                .cube_to_owner
                .sync_with_world_config(&state.lobby.world_config);
            state.save();
            return state.update_clients(&GameLobbyDiff::WorldConfigFull(world_config.clone()));
        }
        McClientToGamelordRequest::WorldConfigRegion(team, region) => {
            let diff = GameLobbyDiff::WorldConfigRegion(team, region);
            let _ = state.lobby.apply_diff(&diff);
            let _ = state
                .cube_to_owner
                .sync_with_world_config(&state.lobby.world_config);
            state.save();
            return state.update_clients(&diff);
        }
    }
}

//have everything handled here
fn handle_kinode_message(
    state: &mut State,
    contract_caller: &mut Option<Caller>,
    ws_channel_id: &mut Option<u32>,
    message: &Message,
) -> anyhow::Result<()> {
    println!("handle kinode message entered");
    if let Ok(request) = serde_json::from_slice::<McClientToGamelordRequest>(&message.body()) {
        println!("Received request: {:?}", request);
        return handle_mcclient_request(state, contract_caller, ws_channel_id, &request, message);
    }
    match GamelordRequestMinecraft::parse(message.body())? {
        GamelordRequestMinecraft::CubeTransitionRequest { minecraft_id, cube } => {
            // we get information about what team the player is in based on Active players
            if let Some(active_player) = state.active_players.get_mut(&minecraft_id) {
                println!(
                    "Player {} is active in the game on team {:?}.",
                    minecraft_id, active_player.team
                );

                // Update the player's current cube
                active_player.current_cube = cube.clone();

                let world_config = &state.lobby.world_config;
                let cube_to_owner = &state.cube_to_owner;

                if let Some(owners) = cube_to_owner.get(&cube) {
                    if !owners.contains(&active_player.team) {
                        // Cube is owned by enemy team(s)
                        for owner in owners {
                            if let Some(team_cubes) = world_config.get(owner) {
                                if let Some(cube_effects) = team_cubes.to_hashmap().get(&cube) {
                                    println!(
                                        "Cube effects for enemy owner {:?}: {:?}",
                                        owner, cube_effects
                                    );
                                    // Send cube effects to mcdriver
                                    let response = serde_json::to_vec(
                                        &GamelordResponseMinecraft::TransitionTriggeredResponse(
                                            cube_effects.clone(),
                                        ),
                                    )
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
                let response =
                    serde_json::to_vec(&GamelordResponseMinecraft::TransitionSilentResponse)
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
        GamelordRequestMinecraft::PlayerSpawnRequest { minecraft_id } => {
            println!(
                "Player spawn request received for player: {:?}",
                minecraft_id
            );

            let team_name = if state
                .lobby
                .team1
                .players
                .iter()
                .any(|p| p.minecraft_player_name == minecraft_id)
            {
                TeamName::Team1
            } else if state
                .lobby
                .team2
                .players
                .iter()
                .any(|p| p.minecraft_player_name == minecraft_id)
            {
                TeamName::Team2
            } else {
                println!("Player {} is not assigned to a team", minecraft_id);
                let response =
                    serde_json::to_vec(&GamelordResponseMinecraft::PlayerSpawnRequestDenied(
                        false,
                        "Player is not assigned to a team.".to_string(),
                    ))
                    .expect("Failed to serialize response");
                Response::new().body(response).send().unwrap();
                return Ok(());
            };

            let spawn_cube = match team_name {
                TeamName::Team1 => &state.lobby.team1.spawn_point,
                TeamName::Team2 => &state.lobby.team2.spawn_point,
            };

            let active_player = ActivePlayer {
                kinode_id: minecraft_id.clone(),
                minecraft_player_name: minecraft_id.clone(),
                current_cube: spawn_cube.clone(),
                team: team_name.clone(),
            };

            state
                .active_players
                .insert(minecraft_id.clone(), active_player);

            println!(
                "Player {} added to active players on team {:?}",
                minecraft_id, &team_name
            );

            let response =
                serde_json::to_vec(&GamelordResponseMinecraft::PlayerSpawnRequestAuthorized(
                    true,
                    format!("Player added to team {:?}.", &team_name),
                    spawn_cube.clone(),
                ))
                .expect("Failed to serialize response");
            Response::new().body(response).send().unwrap();
            Ok(())
        }
        // Think about whether I need this
        GamelordRequestMinecraft::PlayerLeaveRequest { minecraft_id } => {
            if state.active_players.contains_key(&minecraft_id) {
                state.active_players.remove(&minecraft_id);
                println!("Player with kinode_id {} has left the game.", &minecraft_id);
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

fn handle_http_request(
    state: &mut State,
    ws_channel_id: &mut Option<u32>,
    message: &Message,
) -> anyhow::Result<()> {
    let our_http_request = serde_json::from_slice::<http::HttpServerRequest>(message.body())?;
    match our_http_request {
        http::HttpServerRequest::WebSocketOpen { channel_id, .. } => {
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
                    "/api/loadWorld" => load_world(state),
                    "/api/deleteWorld" => {
                        state.lobby.world_config.clear();
                        state.cube_to_owner.clear();
                        state.save();
                        println!("World deleted from request");
                        http::send_response(http::StatusCode::OK, None, b"World Deleted".to_vec());
                    }
                    "/api/clearTeams" => {
                        // need to update clients with lobby with empty teams before actually clearing teams,
                        // because it sends update to team members
                        let _ = state.update_clients(&GameLobbyDiff::Init(
                            state.lobby.clone().clear_teams(),
                        ));
                        state.clear_teams(); // different from lobby.clear_teams
                        state.save();
                        let blob = LazyLoadBlob {
                            mime: Some("application/json".to_string()),
                            bytes: serde_json::to_vec(&GameLobbyDiff::Init(state.lobby.clone()))?,
                        };
                        send_ws_push(ws_channel_id.unwrap_or(0), WsMessageType::Text, blob);
                        http::send_response(http::StatusCode::OK, None, b"Teams Cleared".to_vec());
                    }
                    "/api/editLobby" => edit_lobby(state, ws_channel_id).unwrap_or(()),
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

// used for eth contract testing
fn handle_terminal_message(
    state: &mut State,
    contract_caller: &mut Option<Caller>,
    ws_channel_id: &mut Option<u32>,
    message: &Message,
) -> anyhow::Result<()> {
    println!("terminal message received");

    let action = match serde_json::from_slice::<Action>(&message.body()) {
        Ok(deserialized) => deserialized,
        Err(e) => {
            println!("Failed to deserialize message body: {:?}", e);
            return Ok(());
        }
    };

    match action {
        Action::SetContractAddress(address) => {
            println!("Setting contract address to: {}", address);
            if let PrivateKey::Decrypted(wallet) = state.wallet.clone() {
                *contract_caller = Caller::new(
                    address.as_str(),
                    Provider::new(*CHAIN_ID, 5),
                    *CHAIN_ID,
                    wallet.private_key.as_str(),
                );
            } else {
                return Err(anyhow::anyhow!(
                    "please decrypt the wallet first before proceeding"
                ));
            }
        }
        Action::GetPlayerInfo(funding_address) => {
            if let Some(caller) = contract_caller {
                let result = caller.get_player_info(funding_address);
                println!("result: {:?}", result);
            } else {
                println!("No contract caller found");
            }
        }
        Action::EncryptWallet {
            private_key,
            password,
        } => {
            let private_key = match private_key {
                Some(private_key) => private_key,
                None => {
                    if let PrivateKey::Decrypted(wallet) = state.wallet.clone() {
                        wallet.private_key
                    } else {
                        return Err(anyhow::anyhow!("Private key already encrypted."));
                    }
                }
            };
            let encrypted_wallet_data = encrypt_data(private_key.as_bytes(), password.as_str());
            state.wallet = PrivateKey::Encrypted(encrypted_wallet_data);
            state.save();

            if let Ok(parsed_wallet) = private_key.parse::<LocalWallet>() {
                println!(
                    "Loaded and encrypted wallet with address: {:?}",
                    parsed_wallet.address()
                );
            } else {
                println!("Failed to parse wallet key, try again.");
            }
        }
        Action::DecryptWallet(password) => {
            if let PrivateKey::Encrypted(encrypted_key) = state.wallet.clone() {
                match decrypt_data(&encrypted_key, &password) {
                    Ok(decrypted_key) => match String::from_utf8(decrypted_key)
                        .ok()
                        .and_then(|wd| wd.parse::<LocalWallet>().ok())
                    {
                        Some(parsed_wallet) => {
                            println!(
                                "Decrypted wallet with address: {:?}",
                                parsed_wallet.address()
                            );
                            let serializable_wallet = SerializableWallet::from(parsed_wallet);
                            state.wallet = PrivateKey::Decrypted(serializable_wallet);
                            state.save();
                        }
                        None => println!("Failed to parse wallet, try again."),
                    },
                    Err(_) => println!("Decryption failed, try again."),
                }
            } else {
            }
        } // _ => println!("Invalid message"),
    }
    return Ok(());
}

fn handle_message(
    state: &mut State,
    contract_caller: &mut Option<Caller>,
    ws_channel_id: &mut Option<u32>,
) -> anyhow::Result<()> {
    let message = await_message()?;

    if let "http_server:distro:sys" | "http_client:distro:sys" =
        message.source().process.to_string().as_str()
    {
        println!("HTTP request received.");
        return handle_http_request(state, ws_channel_id, &message);
    }
    if message.is_local(&message.source()) {
        println!("Local message received from: {:?}", message.source());

        if message.source().process.package_name == "terminal" {
            return handle_terminal_message(state, contract_caller, ws_channel_id, &message);
        }
        handle_kinode_message(state, contract_caller, ws_channel_id, &message)?;
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
    // let _ = http::serve_ui(&our, "../mcstructs/ui", true, false, vec!["/"]);
    for path in [
        "/api/loadWorld",
        "/world_config",
        "/api/deleteWorld",
        "/api/editLobby",
        "/api/clearTeams",
    ] {
        http::bind_http_path(path, true, false).expect("failed to bind http path");
    }
    http::bind_http_path("/active_players", false, false).expect("failed to bind http path");
    http::serve_index_html(&our, "ui", true, false, vec!["/"]).unwrap();

    let mut state = State::fetch().unwrap_or_else(|| State::new(&our));

    let mut contract_caller: Option<Caller> = None;
    if let PrivateKey::Decrypted(wallet) = state.wallet.clone() {
        contract_caller = Caller::new(
            "0x5FbDB2315678afecb367f032d93F642f64180aa3",
            Provider::new(*CHAIN_ID, 5),
            *CHAIN_ID,
            &wallet.private_key,
        );
    }

    let min_eth_wager: U256 = "50000000000000000".parse().unwrap(); // 0.05 eth

    loop {
        match handle_message(&mut state, &mut contract_caller, &mut ws_channel_id) {
            Ok(()) => {}
            Err(e) => {
                println!("error from somewhere: {:?}", e);
            }
        };
    }
}
