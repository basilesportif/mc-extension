
use anyhow;
use serde_json;
use chrono::Utc;
use kinode_process_lib::{Address, Message, Request, LazyLoadBlob};
use kinode_process_lib::http::{send_ws_push, WsMessageType};

use crate::{
    State,
    GamelordCaller,
    McClientToGamelordRequest,
    GameLobbyDiff,
    TeamName,
    ChatMessage,
    WorkerRequest,
    MIN_ETH_WAGER,
    Player,
    initialize_worker,
};
use crate::gamelord_types::CubeToOwnerTrait;

use alloy_primitives::Signature;
use std::str::FromStr;

pub fn handle_mcclient_request(
    state: &mut State,
    gamelord_caller: &mut Option<GamelordCaller>,
    ws_channel_id: &mut Option<u32>,
    message: &Message,
    our: &Address,
) -> anyhow::Result<()> {
    if let Ok(request) = serde_json::from_slice::<McClientToGamelordRequest>(&message.body()) {
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
            // Here is where we should handle relaying minecraft .obj, .mtl, and texture files
            McClientToGamelordRequest::JoinTeam(join_team) => {
                println!("got join team");
                let node_id = message.source().node().to_string(); //this is the nodeID I will use
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

                let signature = match Signature::from_str(join_team.signature.as_str()) {
                    Ok(signature) => signature,
                    Err(e) => return Err(anyhow::anyhow!("Error: {}", e)),
                };
                println!("signature: {:?}", signature);
                let recovered_address = signature.recover_address_from_msg(node_id.clone())?;
                if recovered_address != join_team.eth_address {
                    return Err(anyhow::anyhow!("Invalid signature"));
                }
                // get eth wagered and team from chain
                let caller = match gamelord_caller {
                    Some(caller) => caller,
                    None => {
                        println!("no caller");
                        println!("FIX: please use EncryptWallet and then DecryptWallet actions to make caller usable");
                        return Ok(());
                    }
                };

                let (amount_wagered, team) = caller.get_player_info(recovered_address)?;
                if amount_wagered < *MIN_ETH_WAGER {
                    return Err(anyhow::anyhow!("Player has not wagered enough ETH."));
                }

                println!("amount wagered: {:?}", amount_wagered);

                state
                    .node_to_eth
                    .insert(node_id.clone(), (recovered_address, amount_wagered));
                state.save();
                println!("adding to team");
                let _ = add_to_team(state, node_id, join_team.minecraft_id, team, ws_channel_id);

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
            McClientToGamelordRequest::ReadyPlayer(kinode_id) => {
                let diff = GameLobbyDiff::ReadyPlayer(kinode_id.clone());
                let _ = state.lobby.apply_diff(&diff);
                state.save();
                send_ws_push(ws_channel_id.unwrap_or(0), WsMessageType::Text, LazyLoadBlob {
                    mime: Some("application/json".to_string()),
                    bytes: serde_json::to_vec(&diff)?,
                });
                return state.update_clients(&diff);
            }
            McClientToGamelordRequest::RequestFolderMessage { worker_address, folder, encrypt } => {
                let send_dir = format!("{}/pkg/textures", our.package_id());
                let mut current_worker_address = None;
                initialize_worker(our.clone(), &mut current_worker_address)?;
                println!("current worker address: {:?}", current_worker_address);
                let _worker_request = Request::new()
                    .body(serde_json::to_vec(
                        &WorkerRequest::InitializeSenderWorker {
                            target_worker: Some(worker_address),
                            sending_dir: send_dir,
                            password: None
                        },
                    )?
                )
                .target(&current_worker_address.unwrap())
                .send()?;
                return Ok(());
            }
        }
    } else {
        println!("Unhandled request type");
        Ok(())
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