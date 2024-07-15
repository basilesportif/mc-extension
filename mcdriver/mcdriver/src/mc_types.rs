use serde::{Deserialize, Serialize};

// would probaly want to reconfigure this to be more optimal at some point
// actually just the body and outerbody need to be reconfigured

#[derive(Serialize, Deserialize, Debug)]
pub struct WebSocketMessage {
    message_type: String,
    body: MinecraftToGamelord,
}

impl WebSocketMessage {
    pub fn method(&self) -> &MinecraftToGamelord {
        &self.body
    }
}


