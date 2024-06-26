use kinode_process_lib::{
    await_message, call_init, get_blob, http::{self, send_ws_push}, println, Address, LazyLoadBlob, Message, Response
};

use lazy_static::lazy_static;
use std::sync::RwLock;

mod utilities;
use utilities::valid_position;
mod gamelord_types;
use gamelord_types::{Player, Regions, Cube};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

//Here is where we store the CURRENT world config
lazy_static! {
    static ref WORLD_CONFIG: RwLock<HashMap<String, HashMap<u64, Cube>>> = RwLock::new(HashMap::new());
}

lazy_static! {
    static ref WORLD_SHARABLE_CONFIG: RwLock<Regions> = RwLock::new(Regions { regions: Vec::new() });
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
enum GamelordRequest {
    ValidateMove(Player, Cube),
    AddPlayer(Player),
    RemovePlayer(Player),
    GenerateWorld(Regions),
    DeleteWorld
}
impl GamelordRequest {
    fn parse(bytes: &[u8]) -> Result<GamelordRequest, serde_json::Error> {
        serde_json::from_slice::<GamelordRequest>(bytes)
    }
}
// The boolean might not be needed
#[derive(Serialize, Deserialize, Debug)]
enum GamelordResponse {
    ValidateMove(bool, String),
    AddPlayer(bool),
    RemovePlayer(bool),
    WorldGenerated,
    WorldDeleted,
}

#[derive(Debug)]
struct Connection {
    channel_id: u32,
}

wit_bindgen::generate!({
    path: "target/wit",
    world: "process-v0",
});


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
        http::HttpServerRequest::Http(http_request) => {
            match http_request.method().unwrap() {
                http::Method::GET => match http_request.path(){
                    Ok(path) => {
                        match path.as_str() {
                            "/world_config" => {
                                let read_guard = WORLD_SHARABLE_CONFIG.read().unwrap();
                                let sharable_world_config: &Regions = &*read_guard;
                                let serialized_world_config = serde_json::to_string(sharable_world_config).expect("error serializing");
                                http::send_response(
                                    http::StatusCode::OK,
                                    None,
                                    serialized_world_config.into_bytes(),
                                );
                            }
                            _ => {
                                println!("Error handling path");
                                http::send_response(
                                    http::StatusCode::NOT_FOUND,
                                    None,
                                    b"Not Found".to_vec(),
                                );
                            }

                        }
                    }
                    Err(e) => {
                        println!("Error retrieving path: {:?}", e);
                        http::send_response(
                            http::StatusCode::INTERNAL_SERVER_ERROR,
                            None,
                            b"Internal Server Error".to_vec(),
                        );
                    }
                    
                }
                _ => {
                    http::send_response(
                        http::StatusCode::METHOD_NOT_ALLOWED,
                        None,
                        b"Method Not Allowed".to_vec(),
                    );
                }

            }
            
        }
    }
    Ok(())
}


//have everything handled here
fn handle_kinode_message(message: &Message) -> anyhow::Result<()> {
    match GamelordRequest::parse(message.body())? {
        GamelordRequest::GenerateWorld(regions) => {
            let mut world_config = WORLD_CONFIG.write().unwrap();
            world_config.clear();
        
            for region in &regions.regions {
                let owner_cubes = world_config.entry(region.owner.clone()).or_insert_with(HashMap::new);
                for cube in &region.cubes {
                    owner_cubes.insert(cube.identifier(), cube.clone());
                }
            }
            let mut sharable_config = WORLD_SHARABLE_CONFIG.write().unwrap();
            *sharable_config = regions.clone();            

            println!("World generated with regions: {:?}", regions);
            Response::new()
            .body(serde_json::to_vec(&GamelordResponse::WorldGenerated)?)
            .send()
            .unwrap();
            Ok(())
        },
        GamelordRequest::DeleteWorld => {
            let mut world_config = WORLD_CONFIG.write().unwrap();
            world_config.clear();
            let mut sharable_config = WORLD_SHARABLE_CONFIG.write().unwrap();
            sharable_config.regions.clear();

            println!("World deleted");
            Response::new()
            .body(serde_json::to_vec(&GamelordResponse::WorldDeleted)?)
            .send()
            .unwrap();
            Ok(())
        },
        GamelordRequest::ValidateMove(player, cube) => {
            // Placeholder for ValidateMove logic
            Ok(())
        },
        GamelordRequest::AddPlayer(player) => {
            // Placeholder for AddPlayer logic
            Ok(())
        },
        GamelordRequest::RemovePlayer(player) => {
            // Placeholder for RemovePlayer logic
            Ok(())
        },
        
    }
}
    

fn is_websocket_message(message: &Message) -> bool {
    match serde_json::from_slice::<http::HttpServerRequest>(message.body()) {
        Ok(http::HttpServerRequest::WebSocketOpen { .. }) => true,
        Ok(http::HttpServerRequest::WebSocketPush { .. }) => true,
        Ok(http::HttpServerRequest::WebSocketClose { .. }) => true,
        Ok(http::HttpServerRequest::Http{ .. }) => true,
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
    } else if message.is_local(&message.source()) {
        println!("Local message received from: {:?}", message.source());
        handle_kinode_message(&message)?;
    } else{
        println!("Message not handled: {:?}", message);
    }
    Ok(())
}

call_init!(init);
fn init(our: Address) {
    println!("{our}: started");

    http::bind_http_path("/world_config", false, false).expect("failed to bind http path");
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
