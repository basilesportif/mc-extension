use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MCRequest {
    CheckPlayerMove { mc_player_id: String, pos: Position },
}

// python Responses encode the `output` in the `lazy_load_blob`
#[derive(Debug, Serialize, Deserialize)]
pub enum MCResponse {
    PlayerMoveValid { mc_player_id: String, pos: Position },
    PlayerMoveInvalid { mc_player_id: String, pos: Position },
    Err(String),
}
