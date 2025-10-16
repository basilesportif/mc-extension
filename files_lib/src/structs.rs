use kinode_process_lib::Address;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum WorkerRequest {
    InitializeSenderWorker {
        target_worker: Option<Address>,
        sending_dir: String,
        password: Option<String>,
    },
    InitializeReceiverWorker {
        receive_to_dir: String,
    },
    Chunk {
        done: bool,
        file_path: String,
        encrypted: bool,
    },
}

// worker -> main:command_center
#[derive(Serialize, Deserialize, Debug)]
pub enum WorkerStatus {
    Done,
}
