use kinode_process_lib::http;
use kinode_process_lib::kernel_types::MessageType;
use kinode_process_lib::{
    await_message, call_init, get_blob, http::send_ws_push, println, Address, LazyLoadBlob,
    Message, Request, Response,
};

mod mc_types;
use mc_types::{KinodeToMC, MCDriverRequest, MCDriverResponse, MCToKinode};

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
                    let Ok(s) = String::from_utf8(blob.bytes) else {
                        return Ok(());
                    };
                    let Ok(MCToKinode::SanityCheck) = serde_json::from_str(&s) else {
                        println!("got other JSON: {:?}", &s);
                        return Ok(());
                    };
                    send_ws_push(
                        *channel_id,
                        http::WsMessageType::Text,
                        LazyLoadBlob {
                            mime: Some("application/json".to_string()),
                            bytes: serde_json::to_vec(&KinodeToMC::SanityCheckOk).unwrap(),
                        },
                    );
                    println!("Sanity check request received.");
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

    if let Ok(MCDriverRequest::AddPlayer { .. }) = rmp_serde::from_slice(message.body()) {
        println!("AddPlayer request received.");
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
