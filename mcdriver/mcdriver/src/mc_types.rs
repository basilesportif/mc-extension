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

#[derive(Serialize, Deserialize, Debug)]
pub struct Body {
    #[serde(rename = "ValidateMove")]
    pub validate_move: ValidateMove,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OuterBody {
    pub body: Body,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ValidateMove {
    pub player: Player,
    pub cube: Cube,
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
