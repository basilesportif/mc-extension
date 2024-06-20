use kinode_process_lib::{
    await_message, call_init, get_blob,
    println, Address, LazyLoadBlob, Message, http, http::send_ws_push,
};

use lazy_static::lazy_static;
use std::sync::RwLock;


mod utilities;
use utilities::valid_position;
mod gamelord_types;
use gamelord_types::{Player, Regions, Region, Cube};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

//Here is where we store the CURRENT world config
lazy_static! {
    static ref WORLD_CONFIG: RwLock<HashMap<Cube, Region>> = RwLock::new(HashMap::new());
}


#[derive(Deserialize, Debug)]
struct Body {
    player: Player,
    cube: Cube,
}

#[derive(Deserialize, Debug)]
struct OuterBody {
    body: Body,
}
#[derive(Serialize, Deserialize, Debug)]
enum WorldConfig{
    GenerateWorld(Regions),
    DeleteWorld
}
impl WorldConfig {
    fn parse(bytes: &[u8]) -> Result<WorldConfig, serde_json::Error> {
        serde_json::from_slice::<WorldConfig>(bytes)
    }
}

wit_bindgen::generate!({
    path: "target/wit",
    world: "process-v0",
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

fn handle_ws_message(
    connection: &mut Option<Connection>,
    message: Message,
    
) -> anyhow::Result<()> {
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

                    // Load the world configuration
                    let world_config = WORLD_CONFIG.read().unwrap();
                    if world_config.is_empty() {
                        let response = serde_json::to_string(&("world not generated")).unwrap();
                        send_ws_push(
                            *channel_id,
                            http::WsMessageType::Text,
                            LazyLoadBlob {
                                mime: Some("application/json".to_string()),
                                bytes: response.into_bytes(),
                            },
                        );
                        println!("World not generated.");
                        return Ok(());
                    }

                    let body = outer_body.body;
                    let player = body.player;
                    let cube = body.cube;

                    let (response_message, is_valid) = valid_position(&world_config, &player, &cube);

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

fn handle_kinode_message(message: &Message) -> anyhow::Result<()> {
    match WorldConfig::parse(message.body())? {
        WorldConfig::GenerateWorld(regions) => {
            let mut world_config = WORLD_CONFIG.write().unwrap();
            world_config.clear();
            for region in &regions.regions {
                for cube in &region.cubes {
                    world_config.insert(cube.clone(), region.clone());
                }
            }
            println!("World generated with regions: {:?}", regions);
            Ok(())
        }
        WorldConfig::DeleteWorld => {
            let mut world_config = WORLD_CONFIG.write().unwrap();
            world_config.clear();
            println!("World deleted");
            Ok(())
        }
    }
}
    

fn is_websocket_message(message: &Message) -> bool {
    match serde_json::from_slice::<http::HttpServerRequest>(message.body()) {
        Ok(http::HttpServerRequest::WebSocketOpen { .. }) => true,
        Ok(http::HttpServerRequest::WebSocketPush { .. }) => true,
        Ok(http::HttpServerRequest::WebSocketClose { .. }) => true,
        _ => false,
    }
}

fn handle_message(connection: &mut Option<Connection>) -> anyhow::Result<()> {
    let message = await_message()?;
    println!(
        "handle_message: {:?}",
        String::from_utf8_lossy(message.body())
    );

    if is_websocket_message(&message) {
        handle_ws_message(connection, message)?;
    } else {
        handle_kinode_message(&message)?;
    }
    Ok(())
}

call_init!(init);
fn init(our: Address) {
    println!("{our}: started");

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
