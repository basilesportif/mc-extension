use kinode_process_lib::{await_message, call_init, println, Address, Response};
use std::collections::HashMap;

mod gamelord_types;
use gamelord_types::{Cube, Player, Region};

wit_bindgen::generate!({
    path: "target/wit",
    world: "process-v0",
});

//spawn is 986,84,778
// 9 cubes of fake2.dev are surrounding the spawn point
let region_json = serde_json::json!({
    "regions": [
        {
            "cubes": [
                {
                    "center": [950, 100, 750],
                    "radius": 50
                },
            ],
            "owner": "fake.dev"
        },
        {
            "cubes": [
                {
                    "center": [850, 100, 650],
                    "radius": 50
                },
                {
                    "center": [850, 100, 750],
                    "radius": 50
                },
                {
                    "center": [850, 100, 850],
                    "radius": 50
                },
                {
                    "center": [950, 100, 650],
                    "radius": 50
                },
                {
                    "center": [950, 100, 750],
                    "radius": 50
                },
                {
                    "center": [950, 100, 850],
                    "radius": 50
                },
                {
                    "center": [1050, 100, 650],
                    "radius": 50
                },
                {
                    "center": [1050, 100, 750],
                    "radius": 50
                },
                {
                    "center": [1050, 100, 850],
                    "radius": 50
                }
            ],
            "owner": "fake2.dev"
        }
    ]
        }
    ]
});
println!("Region JSON: {region_json}");



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

    for region in regions {
        for cube in &region.cubes {
            cube_to_region_map.insert(cube.clone(), region.clone());
        }
    }
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
