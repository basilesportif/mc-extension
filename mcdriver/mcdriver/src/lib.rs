use kinode_process_lib::http;
use kinode_process_lib::kernel_types::MessageType;
use kinode_process_lib::{
    await_message, call_init, get_blob, println, Address, LazyLoadBlob, Message, Request, Response,
};

mod mc_types;
use mc_types::{MCRequest, MCResponse, Position};

wit_bindgen::generate!({
    path: "wit",
    world: "process"
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

fn handle_ws_message(connection: &mut Option<Connection>, message: Message) -> anyhow::Result<()> {
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
        http::HttpServerRequest::WebSocketPush {
            ref channel_id,
            ref message_type,
        } => {
            if !is_expected_channel_id(connection, channel_id)? {
                // TODO: response?
                return Err(anyhow::anyhow!("d"));
            }
            match message_type {
                http::WsMessageType::Binary => {
                    Response::new()
                        .body(serde_json::to_vec(&MCResponse::PlayerMoveValid {
                            mc_player_id: "".to_string(),
                            pos: Position {
                                x: 0.0,
                                y: 0.0,
                                z: 0.0,
                            },
                        })?)
                        .inherit(true)
                        .send()?;
                }

                http::WsMessageType::Text => {
                    let Some(blob) = get_blob() else {
                        return Ok(());
                    };
                    let Ok(s) = String::from_utf8(blob.bytes) else {
                        return Ok(());
                    };
                    println!("got JSON: {:?}", &s);
                    return Ok(());
                }

                _ => {
                    // TODO: response; handle other types?
                    return Err(anyhow::anyhow!("f"));
                }
            }
        }
    }
    Ok(())
}

fn handle_message(connection: &mut Option<Connection>) -> anyhow::Result<()> {
    let Ok(message) = await_message() else {
        return Ok(());
    };

    println!(
        "handle_message: {:?}",
        String::from_utf8_lossy(message.body())
    );

    if let Ok(MCRequest::CheckPlayerMove { .. }) = rmp_serde::from_slice(message.body()) {
        let Some(Connection { channel_id }) = connection else {
            println!("wrong channel: {:?}", connection);
            panic!("wrong channel");
        };

        Request::new()
            .target("our@http_server:distro:sys".parse::<Address>()?)
            .body(serde_json::to_vec(
                &http::HttpServerAction::WebSocketExtPushOutgoing {
                    channel_id: *channel_id,
                    message_type: http::WsMessageType::Binary,
                    desired_reply_type: MessageType::Response,
                },
            )?)
            .expects_response(15)
            .blob_bytes(message.body())
            //.inherit(true)
            .send()?;
    } else if let Ok(MCRequest::SanityCheck) = rmp_serde::from_slice(message.body()) {
        println!("SanityCheck");
        Response::new()
            .body(serde_json::to_vec(&MCResponse::SanityCheckOk)?)
            .inherit(true)
            .send()?;
    } else {
        handle_ws_message(connection, message)?;
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
