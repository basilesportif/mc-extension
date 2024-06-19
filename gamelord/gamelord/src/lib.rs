use kinode_process_lib::{
    await_message, call_init, get_blob,
    println, Address, LazyLoadBlob, Message, http, http::send_ws_push,
};

mod utilities;
use utilities::valid_position;
mod gamelord_types;
use gamelord_types::{Player, Regions, Region, Cube};
mod fixtures;
use fixtures::get_region_json;
use serde::Deserialize;
use std::collections::HashMap;



#[derive(Deserialize, Debug)]
struct Body {
    player: Player,
    cube: Cube,
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

fn load_regions() -> anyhow::Result<HashMap<Cube, Region>> {
    let config = get_region_json();
    let mut map_config = HashMap::<Cube, Region>::new();
    let map_regions: Regions = serde_json::from_value(config)
        .map_err(|e| {
            println!("Failed to deserialize regions: {:?}", e);
            e
        })?;

    for region in &map_regions.regions {
        for cube in &region.cubes {
            map_config.insert(cube.clone(), region.clone());
        }
    }
    Ok(map_config)
    
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
      regions: &HashMap<Cube, Region>)
       -> anyhow::Result<()> {
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
                    let cube = body.cube;

                    let (response_message, is_valid) = valid_position(regions, &player, &cube);

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
    let regions: HashMap<Cube, Region> = load_regions().unwrap(); 
    handle_ws_message(connection, message, &regions)?;
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
