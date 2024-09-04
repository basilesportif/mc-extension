use kinode_process_lib::{
    get_state, set_state, Address, OnExit, our_capabilities, spawn
};
use mcstructs::GameLobby;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

// spawns a worker process for folder transfer (whether it will be for receiving or sending)
pub fn initialize_worker(
    our: Address,
    current_worker_address: &mut Option<Address>,
) -> anyhow::Result<()> {
    let our_worker = spawn(
        None,
        &format!("{}/pkg/worker.wasm", our.package_id()),
        OnExit::None,
        our_capabilities(),
        vec![],
        false,
    )?;
    // temporarily stores worker address while the worker is alive
    *current_worker_address = Some(Address {
        node: our.node.clone(),
        process: our_worker.clone(),
    });
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub our: Address,
    pub gamelord_address: Option<Address>,
    pub lobby: GameLobby,
}

impl State {
    pub fn new(our: &Address) -> Self {
        State {
            our: our.clone(),
            gamelord_address: None,
            lobby: GameLobby::new(),
        }
    }
    pub fn fetch() -> Option<State> {
        if let Some(state_bytes) = get_state() {
            bincode::deserialize(&state_bytes).ok()
        } else {
            None
        }
    }
    pub fn save(&self) {
        let serialized_state = bincode::serialize(self).expect("Failed to serialize state");
        set_state(&serialized_state);
    }
}