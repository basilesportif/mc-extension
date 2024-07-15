use kinode_process_lib::{
    await_message, call_init, get_blob, get_state, http, println, set_state, Address, NodeId,
    Request,
};
use mcstructs::{GameLobby, GameLobbyDiff, JoinTeam, McClientToGamelordRequest};
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

fn handle_message(state: &mut State) -> anyhow::Result<()> {
    let message = await_message()?;

    if let Some(gamelord) = &state.gamelord_address {
        if message.source() == gamelord {
            let deserialized = serde_json::from_slice::<GameLobbyDiff>(message.body())?;
            match deserialized.clone() {
                GameLobbyDiff::Init(..) => {
                    state.lobby = state.lobby.apply_diff(deserialized);
                    state.save();
                    println!("received init");
                }
                GameLobbyDiff::AddPlayerToTeam(..) => {
                    state.lobby = state.lobby.apply_diff(deserialized);
                    state.save();
                    println!("received add player to team");
                }
            }
            return Ok(());
        }
    }

    if message.source().node() == state.our.node() {
        return handle_http_request(state, message.body());
    }

    Ok(())
}

fn handle_http_request(state: &mut State, body: &[u8]) -> anyhow::Result<()> {
    let http_request = http::HttpServerRequest::from_bytes(body)?;
    let http_request = http_request
        .request()
        .ok_or_else(|| anyhow::anyhow!("Failed to parse http request"))?;
    let path = http_request.path()?;
    let bytes = get_blob()
        .ok_or_else(|| anyhow::anyhow!("Failed to get blob"))?
        .bytes;

    match path.as_str() {
        "/join_team" => {
            let ui_request: JoinTeam = serde_json::from_slice(&bytes)?;
            // println!("mcclient: {:#?}", ui_request);

            let gamelord = Address::new(
                ui_request.gamelord_id.clone(),
                ("gamelord", "gamelord", "basilesex.os"),
            );
            let join_request =
                serde_json::to_vec(&McClientToGamelordRequest::JoinTeam(ui_request.clone()))?;
            let _ = Request::to(gamelord.clone())
                .body(join_request)
                .send();
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
        _ => Ok(()),
    }
}

call_init!(init);
fn init(our: Address) {
    println!("start mcclient");
    let mut state = State::fetch().unwrap_or_else(|| State::new(&our));

    let _ = http::serve_ui(&our, "ui", true, false, vec!["/"]);

    for path in ["/join_team"] {
        http::bind_http_path(path, true, false).expect("failed to bind http path");
    }

    http::serve_index_html(&our, "ui", true, false, vec!["/"]).unwrap_or_default();

    if let Some(gamelord_address) = state.clone().gamelord_address {
        let _ = Request::to(gamelord_address)
            .body(serde_json::to_vec(&McClientToGamelordRequest::Init).unwrap())
            .send();
    }

    loop {
        match handle_message(&mut state) {
            Ok(_) => {}
            Err(e) => {
                println!("mcclient: error: {:?}", e);
            }
        };
    }
}
