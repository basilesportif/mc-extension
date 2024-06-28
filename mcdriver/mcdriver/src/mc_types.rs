use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

//TODO delete all this
#[derive(Debug, Serialize, Deserialize)]
pub enum MCDriverRequest {
    AddPlayer { mc_player_id: String },
}

// TODO: delete all this
// python Responses encode the `output` in the `lazy_load_blob`
#[derive(Debug, Serialize, Deserialize)]
pub enum MCDriverResponse {
    Ok,
    Err(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MCToKinode {
    SanityCheck,
    SanityCheckErr(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum KinodeToMC {
    SanityCheckOk,
    SanityCheckErr(String),
}

// would probaly want to reconfigure this to be more optimal at some point
// actually just the body and outerbody need to be reconfigured
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Player {
    pub kinode_id: String,
    pub minecraft_player_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Cube {
    pub center: (i32, i32, i32),
    pub side_length: i32,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValidateMove {
    player: Player,
    cube: Cube,
}

impl ValidateMove {
    pub fn player(&self) -> &Player {
        &self.player
    }
    pub fn cube(&self) -> &Cube {
        &self.cube
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerJoinRequest {
    player: Player,
}

impl PlayerJoinRequest {
    pub fn player(&self) -> &Player {
        &self.player
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Method {
    ValidateMove { ValidateMove: ValidateMove },
    PlayerJoinRequest { PlayerJoinRequest: PlayerJoinRequest },
    // Add other message types here
}

impl Method {
    pub fn as_validate_move(&self) -> Option<&ValidateMove> {
        if let Method::ValidateMove { ValidateMove } = self {
            Some(ValidateMove)
        } else {
            None
        }
    }

    pub fn as_player_join(&self) -> Option<&PlayerJoinRequest> {
        if let Method::PlayerJoinRequest { PlayerJoinRequest } = self {
            Some(PlayerJoinRequest)
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebSocketMessage {
    message_type: String,
    body: Method,
}

impl WebSocketMessage {
    pub fn message_type(&self) -> &String {
        &self.message_type
    }

    pub fn method(&self) -> &Method {
        &self.body
    }
}

/*
Brainstorm types:
- from Minecraft
 * new player id joined
 * should this player be allowed?
 * can this player move to this spot (request id and spot)
 * can this player take ownership of this spot?
- to Minecraft
 * player join allowed/disallowed (request uuid)
 * player move allowed/disallowed (request uuid)
 * player take ownership allowed/disallowed (request uuid)
*/
