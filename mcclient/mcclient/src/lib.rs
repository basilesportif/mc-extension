use kinode_process_lib::http::{bind_ws_path, send_ws_push, WsMessageType};
use kinode_process_lib::{
    await_message, call_init, get_blob, get_state, http, println, set_state, Address, LazyLoadBlob,
    Request,
};
use kinode_process_lib::NodeId;
use mcstructs::{
    Cube, CubeEffectList, GameLobby, GameLobbyDiff, JoinTeam, McClientToGamelordRequest, Region, AssociateMinecraftId,
    WsPush,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

wit_bindgen::generate!({
    path: "target/wit",
    world: "process-v0",
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub our: Address,
    pub gamelord_address: Option<Address>,
    pub lobby: GameLobby,
}

impl State {
    pub fn new(our: &Address) -> Self {
        State {
            our: our.clone(),
            gamelord_address: None,
            lobby: GameLobby::new(),
        }
    }
    pub fn fetch() -> Option<State> {
        if let Some(state_bytes) = get_state() {
            bincode::deserialize(&state_bytes).ok()
        } else {
            None
        }
    }
    pub fn save(&self) {
        let serialized_state = bincode::serialize(self).expect("Failed to serialize state");
        set_state(&serialized_state);
    }
}

fn handle_http_request(
    state: &mut State,
    ws_channel_id: &mut Option<u32>,
    body: &[u8],
    our: &Address,
) -> anyhow::Result<()> {
    let http_request = http::HttpServerRequest::from_bytes(body)?;

    match http_request {
        http::HttpServerRequest::WebSocketOpen { channel_id, .. } => {
            *ws_channel_id = Some(channel_id);
            send_ws_push(
                channel_id,
                WsMessageType::Text,
                LazyLoadBlob {
                    mime: Some("application/json".to_string()),
                    bytes: serde_json::to_vec(&GameLobbyDiff::Init(state.lobby.clone()))?,
                },
            );
            send_ws_push(
                channel_id,
                WsMessageType::Text,
                LazyLoadBlob {
                    mime: Some("application/json".to_string()),
                    bytes: serde_json::to_vec(&serde_json::json!({"OurNode": state.our.node()}))?,
                },
            );
            return Ok(());
        }
        http::HttpServerRequest::WebSocketClose { .. } => {
            *ws_channel_id = None;
            return Ok(());
        }
        http::HttpServerRequest::WebSocketPush { .. } => {
            let Some(blob) = get_blob() else {
                return Ok(());
            };
            let ws_push = serde_json::from_slice::<WsPush>(&blob.bytes)?;
            match ws_push {
                WsPush::SendMessage(message) => {
                    // println!("mcclient: received message: {}", message);
                    if let Some(gamelord) = &state.gamelord_address {
                        let msg_request: Vec<u8> = serde_json::to_vec(
                            &McClientToGamelordRequest::SendMessage(message.clone()),
                        )?;
                        let _ = Request::to(gamelord.clone()).body(msg_request).send();
                    } else {
                        return Err(anyhow::anyhow!("mcclient: no gamelord address"));
                    }
                }
                WsPush::WorldConfigRegion(team, region) => {
                    if let Some(gamelord) = &state.gamelord_address {
                        let update_region_request =
                            McClientToGamelordRequest::WorldConfigRegion(team, region);
                        let _ = Request::to(gamelord.clone())
                            .body(serde_json::to_vec(&update_region_request)?)
                            .send();
                    }
                }
                _ => {}
            }
            return Ok(());
        }
        _ => {}
    }

    let http_request = http_request
        .request()
        .ok_or_else(|| anyhow::anyhow!("Failed to parse http request"))?;
    let path = http_request.path()?;
    let bytes = get_blob()
        .ok_or_else(|| anyhow::anyhow!("Failed to get blob"))?
        .bytes;

    match path.as_str() {
        "/join_team" => {
            println!("http request: join_team");

            let ui_request: JoinTeam = serde_json::from_slice(&bytes)?;
            // println!("mcclient: {:#?}", ui_request);

            let gamelord = Address::new(
                ui_request.gamelord_id.clone(),
                ("gamelord", "gamelord", "basilesex.os"),
            );
            let join_request =
                serde_json::to_vec(&McClientToGamelordRequest::JoinTeam(ui_request.clone()))?;
            let _ = Request::to(gamelord.clone()).body(join_request).send();
            let _ = Request::to(gamelord.clone())
                .body(serde_json::to_vec(&McClientToGamelordRequest::Init).unwrap())
                .send();

            state.gamelord_address = Some(gamelord);
            state.save();

            http::send_response(
                http::StatusCode::OK,
                Some(HashMap::from([(
                    "Content-Type".to_string(),
                    "application/json".to_string(),
                )])),
                b"{\"message\": \"success\"}".to_vec(),
            );
            Ok(())
        }
        "/ready_player" => {
            println!("http request: ready_player");

            let ui_request: NodeId = serde_json::from_slice(&bytes)?;
            let gamelord = state.gamelord_address.clone().ok_or_else(|| anyhow::anyhow!("No gamelord address"))?;
            let gamelord = Address::new(
                gamelord.node(),    
                ("gamelord", "gamelord", "basilesex.os"),
            );
            let _ = Request::to(gamelord.clone())
                .body(serde_json::to_vec(&McClientToGamelordRequest::ReadyPlayer(ui_request.clone()))?)
                .send();
            
            // Save the state
            state.save();

            http::send_response(
                http::StatusCode::OK,
                Some(HashMap::from([(
                    "Content-Type".to_string(),
                    "application/json".to_string(),
                )])),
                b"{\"message\": \"success\"}".to_vec(),
            );
            Ok(())
        }
        //TO DO: send to gamelord
        "/associate_minecraft_id" => {
            let ui_request: AssociateMinecraftId = serde_json::from_slice(&bytes)?;
            let gamelord = Address::new(
                ui_request.gamelord_id.clone(),
                ("gamelord", "gamelord", "basilesex.os"),
            );
            let _ = Request::to(gamelord.clone())
                .body(serde_json::to_vec(&McClientToGamelordRequest::AssociateMinecraftId(ui_request.clone()))?)
                .send();
            
            http::send_response(
                http::StatusCode::OK,
                Some(HashMap::from([(
                    "Content-Type".to_string(),
                    "application/json".to_string(),
                )])),
                b"{\"message\": \"success\"}".to_vec(),
            );
            Ok(())
        }
        _ => {
            println!("mcclient: unknown http request: {:?}", path);
            Ok(())
        }
    }
}

fn handle_gamelord_update(
    state: &mut State,
    ws_channel_id: &mut Option<u32>,
    body: &[u8],
) -> anyhow::Result<()> {
    println!("received update from gamelord");
    let deserialized = serde_json::from_slice::<GameLobbyDiff>(body)?;
    println!("deserialized update: {:#?}", deserialized);
    state.lobby = match state.lobby.apply_diff(&deserialized) {
        Ok(lobby) => lobby,
        Err(e) => {
            println!("mcclient: error applying diff: {}", e);
            return Ok(());
        }
    };
    println!("state: {:#?}", state.lobby);
    state.save();
    let blob = LazyLoadBlob {
        mime: Some("application/json".to_string()),
        bytes: serde_json::to_vec(&deserialized)?,
    };
    send_ws_push(ws_channel_id.unwrap_or(0), WsMessageType::Text, blob);
    Ok(())
}

fn handle_message(
    state: &mut State,
    ws_channel_id: &mut Option<u32>,
    our: &Address,
) -> anyhow::Result<()> {
    let message = await_message()?;

    if let Some(gamelord) = &state.gamelord_address {
        if message.source() == gamelord {
            return handle_gamelord_update(state, ws_channel_id, message.body());
        }
    }

    if message.source().node() == state.our.node() {
        return handle_http_request(state, ws_channel_id, message.body(), our);
    }

    Ok(())
}

call_init!(init);
fn init(our: Address) {
    println!("start mcclient");
    let mut ws_channel_id: Option<u32> = None;
    bind_ws_path("/", true, false).unwrap();

    let _ = http::serve_ui(&our, "ui", true, false, vec!["/"]);
    for path in ["/join_team", "/ready_player"] {
        http::bind_http_path(path, true, false).expect("failed to bind http path");
    }
    http::serve_index_html(&our, "ui", true, false, vec!["/"]).unwrap_or_default();

    let mut state: State = State::fetch().unwrap_or_else(|| State::new(&our));

    if let Some(gamelord_address) = state.gamelord_address.clone() {
        let _ = Request::to(gamelord_address)
            .body(serde_json::to_vec(&McClientToGamelordRequest::Init).unwrap())
            .send();
    }

    loop {
        match handle_message(&mut state, &mut ws_channel_id, &our) {
            Ok(_) => {}
            Err(e) => {
                println!("mcclient: error: {:?}", e);
            }
        };
    }
}