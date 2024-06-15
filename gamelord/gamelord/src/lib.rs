use kinode_process_lib::{await_message, call_init, println, Address, Response};
use std::collections::HashMap;

mod gamelord_types;
use gamelord_types::{Cube, Player, Region};
mod fixtures;
use fixtures::get_region_json;

wit_bindgen::generate!({
    path: "target/wit",
    world: "process-v0",
});

fn handle_message(_our: &Address) -> anyhow::Result<()> {
    let message = await_message()?;

    if !message.is_request() {
        return Err(anyhow::anyhow!("unexpected Response: {:?}", message));
    }

    let body: serde_json::Value = serde_json::from_slice(message.body())?;
    println!("got {body:?}");
    Response::new()
        .body(serde_json::to_vec(&serde_json::json!("Ack")).unwrap())
        .send()
        .unwrap();
    Ok(())
}

call_init!(init);
fn init(our: Address) {
    let mut cube_to_region_map: HashMap<Cube, Region> = HashMap::new();
    // Assuming regions are predefined or loaded from some source
    let regions: Vec<Region> = vec![]; // This should be populated appropriately in real use

    /*
    for region in regions {
        for cube in &region.cubes {
            cube_to_region_map.insert(cube.clone(), region.clone());
        }
    }
    */
    println!("begin");

    loop {
        match handle_message(&our) {
            Ok(()) => {}
            Err(e) => {
                println!("error: {:?}", e);
            }
        };
    }
}
