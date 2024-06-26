use std::str::FromStr;
use kinode_process_lib::{http, ProcessId};
use serde::{Deserialize, Serialize};
use kinode_process_lib::kernel_types::MessageType;
use kinode_process_lib::{
    await_message, call_init, get_blob, http::send_ws_push, println, Address, LazyLoadBlob,
    Message, Request, Response,
};

mod mc_types;
use mc_types::{KinodeToMC, MCDriverRequest, MCDriverResponse, MCToKinode};

#[derive(Serialize, Deserialize, Debug)]
struct Body {
    #[serde(rename = "ValidateMove")]
    validate_move: ValidateMove,
}

#[derive(Serialize, Deserialize, Debug)]
struct OuterBody {
    body: Body,
}

#[derive(Serialize, Deserialize, Debug)]
struct ValidateMove {
    player: Player,
    cube: Cube,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Player {
    pub kinode_id: String,
    pub minecraft_player_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Cube {
    pub center: (i32, i32, i32),
    pub side_length: i32,
}

wit_bindgen::generate!({
    path: "target/wit",
    world: "process-v0"
});

#[derive(Debug)]
struct Connection {
    channel_id: u32,
}

fn is_expected_channel_id(
    connection: &Option<Connection>,
    channel_id: &u32,
) -> anyhow::Result<bool> {
    let Some(Connection {
        channel_id: ref current_channel_id,
    }) = connection
    else {
        return Err(anyhow::anyhow!("a"));
    };

    Ok(channel_id == current_channel_id)
}


fn process_request(player: &Player, cube: &Cube) -> anyhow::Result<serde_json::Value> {
    println!("Processing request for player: {:?}", player);
    let action = serde_json::json!({
        "ValidateMove": [
            player,  // First element of the tuple
            cube     // Second element of the tuple
        ]
    });
    let response = Request::new()
        .target(Address::new("fake.dev", ProcessId::from_str("gamelord:gamelord:basilesex.os").unwrap()))
        .body(serde_json::to_vec(&action)?)
        .send_and_await_response(1);

    match response {
        Ok(message) => {
            let body = serde_json::from_slice::<serde_json::Value>(&message.unwrap().body())
                .expect("Failed to parse response body as JSON");
            return Ok(body);
        },
        Err(e) => {
            println!("Failed to send or receive response: {:?}", e);
            return Err(e);
        }
    }
}


fn handle_ws_message(
    connection: &mut Option<Connection>,
     message: Message,
    ) -> anyhow::Result<()> {
    match serde_json::from_slice::<http::HttpServerRequest>(message.body())? {
        http::HttpServerRequest::Http(_) => {
            // TODO: response?
            return Err(anyhow::anyhow!("b"));
        }
        http::HttpServerRequest::WebSocketOpen { channel_id, .. } => {
            println!("WSOPEN channel open: {}", channel_id);
            *connection = Some(Connection { channel_id });
        }
        http::HttpServerRequest::WebSocketClose(ref channel_id) => {
            if !is_expected_channel_id(connection, channel_id)? {
                // TODO: response?
                return Err(anyhow::anyhow!("c"));
            }
            *connection = None;
        }
        /* if we get a push message, process Binary and Text versions of it */
        http::HttpServerRequest::WebSocketPush {
            ref channel_id,
            ref message_type,
        } => {
            if !is_expected_channel_id(connection, channel_id)? {
                // TODO: response?
                return Err(anyhow::anyhow!("d"));
            }
            match message_type {
                http::WsMessageType::Text => {
                    let Some(blob) = get_blob() else {
                        return Ok(());
                    };
                    println!("Received blob: {:?}", String::from_utf8_lossy(&blob.bytes));

                    let outer_body: OuterBody = match serde_json::from_slice(&blob.bytes) {
                        Ok(outer_body) => outer_body,
                        Err(e) => {
                            println!("Invalid JSON: {:?}", e);
                            return Ok(());
                        }
                    };

                    // Since player and cube are always present, directly access them
                    let player = &outer_body.body.validate_move.player;
                    let cube = &outer_body.body.validate_move.cube;
                    let outcome = process_request(player, cube)?;
                    let serialized_message = serde_json::to_string(&outcome).expect("Failed to serialize JSON");

                    send_ws_push(
                        *channel_id,
                        http::WsMessageType::Text,
                        LazyLoadBlob {
                            mime: Some("application/json".to_string()),
                            bytes: serialized_message.into_bytes(),
                        },
                    );
                    println!("Position check request received.");
                    return Ok(());
                }

                _ => {
                    return Err(anyhow::anyhow!("Unsupported message type"));
                }
            }
        }
    }
    Ok(())
}


fn handle_message(connection: &mut Option<Connection>) -> anyhow::Result<()> {
    let message = await_message()?;

    println!(
        "handle_message: {:?}",
        String::from_utf8_lossy(message.body())
    );
    // just for now, we'll probably have a different method for authentication
    if message.is_local(&message.source()) {
        println!("Local message received.");
        handle_ws_message(connection, message)?;
        //handle_local_message(&message);
    } else {
        // Will handle this better, wanted to keep your code
        if let Ok(MCDriverRequest::AddPlayer { .. }) = rmp_serde::from_slice(message.body()) {
            println!("AddPlayer request received.");
        } else {
            println!("WS message received.");
            handle_ws_message(connection, message)?;
        }
    }

    Ok(())
}

call_init!(init);
fn init(our: Address) {
    println!("{our}: begin");

    let mut connection: Option<Connection> = None;
    http::bind_ext_path("/").unwrap();

    loop {
        match handle_message(&mut connection) {
            Ok(()) => {}
            Err(e) => {
                println!("{our}: error: {:?}", e);
            }
        };
    }
}
