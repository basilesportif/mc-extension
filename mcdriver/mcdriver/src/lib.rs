use std::str::FromStr;
use kinode_process_lib::{http, ProcessId};
use serde::{Deserialize, Serialize};
use kinode_process_lib::kernel_types::MessageType;
use kinode_process_lib::{
    await_message, call_init, get_blob, http::send_ws_push, println, Address, LazyLoadBlob,
    Message, Request,
};

mod mc_types;
use mc_types::{WebSocketMessage, MinecraftToGamelord};


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

//going to Kinode
fn process_gamelord_request(minecraft_id: &String, method: &MinecraftToGamelord) -> anyhow::Result<serde_json::Value> {
    println!("Processing request for player: {:?}", minecraft_id);
    let action = match method {
        MinecraftToGamelord::ValidateMove { .. } => method,
        MinecraftToGamelord::PlayerSpawnRequest { .. } => method,
        _ => return Err(anyhow::anyhow!("Unsupported request type")),
    };

    let response = Request::new()
        .target(Address::new("fake.dev", ProcessId::from_str("gamelord:gamelord:basilesex.os").unwrap()))
        .body(serde_json::to_vec(&action)?)
        .send_and_await_response(5);

    match response {
        Ok(message) => {
            let body = match message {
                Ok(msg) => serde_json::from_slice::<serde_json::Value>(msg.body())
                    .map_err(|e| anyhow::anyhow!("Failed to parse response body: {}", e))?,
                Err(e) => return Err(anyhow::anyhow!("Failed to receive response: {}", e)),
            };
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
            send_ws_push(
                channel_id,
                http::WsMessageType::Text,
                LazyLoadBlob {
                    mime: Some("text/plain".to_string()),
                    bytes: "Connection established".as_bytes().to_vec(),
                },
            );
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

                    let ws_message: WebSocketMessage = match serde_json::from_slice(&blob.bytes) {
                        Ok(ws_message) => ws_message,
                        Err(e) => {
                            println!("Invalid JSON: {:?}", e);
                            return Ok(());
                        }
                    };

                    match ws_message.method() {
                        MinecraftToGamelord::ValidateMove {  minecraft_id, cube } => {
                            let outcome = process_gamelord_request(minecraft_id,
                                                          &MinecraftToGamelord::ValidateMove { 
                                                              minecraft_id: minecraft_id.to_string(), 
                                                              cube: cube.clone() 
                                                          })?;
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
                        }
                        MinecraftToGamelord::PlayerSpawnRequest { minecraft_id } => {
                            let outcome = process_gamelord_request(minecraft_id,
                                                            &MinecraftToGamelord::PlayerSpawnRequest { 
                                                                minecraft_id: minecraft_id.to_string()
                                                            })?;
                            let serialized_message = serde_json::to_string(&outcome).expect("Failed to serialize JSON");

                            send_ws_push(
                                *channel_id,
                                http::WsMessageType::Text,
                                LazyLoadBlob {
                                    mime: Some("application/json".to_string()),
                                    bytes: serialized_message.into_bytes(),
                                },
                            );
                            println!("Player join request received for player: {:?}", &minecraft_id);
                        }
                        // Add other message types here
                    }
                    return Ok(());
                }
                http::WsMessageType::Ping => {
                    send_ws_push(
                        *channel_id,
                        http::WsMessageType::Pong,
                        LazyLoadBlob {
                            mime: None,
                            bytes: vec![],
                        },
                    );
                    println!("Ping received, pong sent.");
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
        "handle_message: {:?}, {:?}",
        String::from_utf8_lossy(message.body()),
        message.source()
    );
    // just for now, we'll probably have a different method for authentication
    if message.is_local(&message.source()) {
        println!("Local message received.");
        handle_ws_message(connection, message)?;
    } else {
        println!("Invalid message"); 
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
