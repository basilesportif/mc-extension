use std::str::FromStr;
use kinode_process_lib::{http, ProcessId};
use kinode_process_lib::{
    await_message, call_init, get_blob, http::send_ws_push, println, Address, LazyLoadBlob,
    Message, Request,};
use serde_json::Value;

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

// The body of the request is deserialized by Gamelord, this function just relays the request to the gamelord
// and returns the response from Gamelord
fn process_gamelord_request(request: &[u8]) -> anyhow::Result<Vec<u8>> {
    let response = Request::new()
        .target(Address::new("uncentered-gamelord.os", ProcessId::from_str("gamelord:gamelord:basilesex.os").unwrap()))
        .body(request.to_vec())
        .send_and_await_response(2)?;
        match response {
            Ok(msg) => Ok(msg.body().to_vec()),
            Err(e) => Err(anyhow::anyhow!("Failed to receive response: {}", e)),
        }
}

//
fn handle_ws_message(
    connection: &mut Option<Connection>,
     message: Message,
    ) -> anyhow::Result<()> {
    match serde_json::from_slice::<http::HttpServerRequest>(message.body())? {
        // 
        http::HttpServerRequest::Http(_) => {
            http::send_response(
                http::StatusCode::BAD_REQUEST,
                None,
                b"Should not be receiving HTTP requests".to_vec(),
            );
            return Err(anyhow::anyhow!("b"));
        }
        // Send message to the MC plugin that a connection has been established
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
        // Should probably handle cases where the WS connection is closed, but not a priority right now
        http::HttpServerRequest::WebSocketClose(ref channel_id) => {
            if !is_expected_channel_id(connection, channel_id)? {
                // TODO: response?
                return Err(anyhow::anyhow!("c"));
            }
            *connection = None;
        }
        // Relays messages from the MC plugin to the gamelord (both ways <-/->)
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
                    // probably not the best way to do this, should come back to it
                    // Parse the JSON and extract only the body (which removes the WS metadata and converts it to gamelord)
                    let parsed: Value = serde_json::from_slice(&blob.bytes)?;
                    // parse the `body` of the WS message (which should contain gamelord stuff)
                    if let Some(body) = parsed.get("body") {
                        let body_json = serde_json::to_vec(body)?;
                        // println!("Extracted body: {}", String::from_utf8_lossy(&body_json));

                        // forward only the body to gamelord
                        match process_gamelord_request(&body_json) {
                            Ok(response) => {
                                send_ws_push(
                                    *channel_id,
                                    http::WsMessageType::Text,
                                    LazyLoadBlob {
                                        mime: Some("application/json".to_string()),
                                        bytes: response,
                                    },
                                );
                                println!("Message relayed to gamelord.");
                            },
                            Err(e) => {
                                println!("Error processing request: {:?}", e);
                            }
                        }
                    } else {
                        println!("No body found in the message");
                    }
                    return Ok(());
                }
                // Respond with a pong to the MC server
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
    // Assumption here is that mcdriver and gamelord are on the same node
    if message.is_local(&message.source()) {
        handle_ws_message(connection, message)?;
    } else {
        println!("Message source is not from the same node"); 
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
