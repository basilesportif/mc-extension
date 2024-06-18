use kinode_process_lib::{
    await_message, call_init, get_blob,
    println, Address, LazyLoadBlob, Message, http, http::send_ws_push,
};

mod utilities;
use utilities::valid_position;
mod gamelord_types;
use gamelord_types::{Player, Regions, CurrentPosition};
mod fixtures;
use fixtures::get_region_json;
use serde::Deserialize;



#[derive(Deserialize, Debug)]
struct Body {
    player: Player,
    position: CurrentPosition,
}

#[derive(Deserialize, Debug)]
struct OuterBody {
    body: Body,
}

wit_bindgen::generate!({
    path: "target/wit",
    world: "process-v0",
});

#[derive(Debug)]
struct Connection {
    channel_id: u32,
}

fn load_regions() -> anyhow::Result<Regions> {
    let config = get_region_json();
    let map_regions: Regions = serde_json::from_value(config)
        .map_err(|e| {
            println!("Failed to deserialize regions: {:?}", e);
            e
        })?;
    Ok(map_regions)
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
        http::HttpServerRequest::WebSocketOpen { channel_id, .. } => {
            println!("WSOPEN channel open: {}", channel_id);
            *connection = Some(Connection { channel_id });
        }
        http::HttpServerRequest::WebSocketPush {
            ref channel_id,
            ref message_type,
        } => {
            if !is_expected_channel_id(connection, channel_id)? {
                return Err(anyhow::anyhow!("Unexpected channel ID"));
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
                    // probably not good
                    let body = outer_body.body;
                    let player = body.player;
                    let position = body.position;

                    let regions = load_regions().unwrap(); // Assume this function fetches the current regions
                    let is_valid = valid_position(&regions, &player, &position);
                    let response_message = if is_valid {
                        "Player is allowed in this position"
                    } else {
                        "Player is not allowed in this position"
                    };

                    let response = serde_json::to_string(&(is_valid, response_message)).unwrap();

                    send_ws_push(
                        *channel_id,
                        http::WsMessageType::Text,
                        LazyLoadBlob {
                            mime: Some("application/json".to_string()),
                            bytes: response.into_bytes(),
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

        http::HttpServerRequest::WebSocketClose(ref channel_id) => {
            if !is_expected_channel_id(connection, channel_id)? {
                return Err(anyhow::anyhow!("Unexpected channel ID"));
            }
            *connection = None;
        }

        http::HttpServerRequest::Http(_) => {
            return Err(anyhow::anyhow!("Unexpected HTTP request"));
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
    handle_ws_message(connection, message)?;
    Ok(())
}

call_init!(init);
fn init(our: Address) {
    println!("{our}: started");

    // Assuming regions are predefined or loaded from some source
    let config = get_region_json();
    let map_regions: Regions = match serde_json::from_value(config) {
        Ok(regions) => regions,
        Err(e) => {
            println!("Failed to deserialize regions: {:?}", e);
            return;
        }
    };
    //need to encode this info into 
    let player = Player {
        kinode_id: "fake.dev".to_string(),
        minecraft_player_name: "player2".to_string(),
    };
    let position = CurrentPosition { x: 1050, y: 100, z: 750 };

    let output = valid_position(&map_regions, &player, &position);
    println!("The action is allowed?: {output}");
    http::bind_ext_path("/").unwrap();
    println!("begin");
    let mut connection: Option<Connection> = None;
    loop {
        match handle_message(&mut connection) {
            Ok(()) => {}
            Err(e) => {
                println!("error: {:?}", e);
            }
        };
    }
    
}
