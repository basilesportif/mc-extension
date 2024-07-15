use kinode_process_lib::{
    await_message, call_init, get_blob, http, println, Address, NodeId, Request,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use mcstructs::{JoinTeam, McClientToGamelordRequest};

wit_bindgen::generate!({
    path: "target/wit",
    world: "process-v0",
});

fn handle_message(our: &Address) -> anyhow::Result<()> {
    let message = await_message()?;
    handle_http_request(our, message.body())
}

fn handle_http_request(our: &Address, body: &[u8]) -> anyhow::Result<()> {
    let http_request = http::HttpServerRequest::from_bytes(body)?;
    let http_request = http_request
        .request()
        .ok_or_else(|| anyhow::anyhow!("Failed to parse http request"))?;
    let path = http_request.path()?;
    let bytes = get_blob()
        .ok_or_else(|| anyhow::anyhow!("Failed to get blob"))?
        .bytes;

    match path.as_str() {
        "/join_team" => {
            let ui_request: JoinTeam = serde_json::from_slice(&bytes)?;
            println!("mcclient: {:?}", ui_request);

            let join_request =
                serde_json::to_vec(&McClientToGamelordRequest::JoinTeam(ui_request.clone()))?;
            let _ = Request::to(Address::new(
                ui_request.gamelord_id,
                ("gamelord", "gamelord", "basilesex.os"),
            ))
            .expects_response(5)
            .body(join_request)
            .send_and_await_response(5);

            http::send_response(
                http::StatusCode::OK,
                Some(HashMap::from([(
                    "Content-Type".to_string(),
                    "application/json".to_string(),
                )])),
                b"{\"message\": \"success\"}".to_vec(),
            );
            Ok(())
        }
        _ => Ok(()),
    }
}

call_init!(init);
fn init(our: Address) {
    println!("start mcclient");

    let _ = http::serve_ui(&our, "ui", true, false, vec!["/"]);

    for path in ["/join_team"] {
        http::bind_http_path(path, true, false).expect("failed to bind http path");
    }

    http::serve_index_html(&our, "ui", true, false, vec!["/"]).unwrap_or_default();

    loop {
        match handle_message(&our) {
            Ok(_) => {}
            Err(e) => {
                println!("mcclient: error: {:?}", e);
            }
        };
    }
}
