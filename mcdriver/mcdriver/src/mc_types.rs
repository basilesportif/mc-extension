use serde::{Deserialize, Serialize};

// would probaly want to reconfigure this to be more optimal at some point
// actually just the body and outerbody need to be reconfigured

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Cube {
    pub center: (i32, i32, i32),
    pub side_length: i32,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MinecraftToGamelord {
    ValidateMove {
        minecraft_id: String,
        cube: Cube,
    },
    PlayerSpawnRequest {
        minecraft_id: String,
    },
    // Add other message types here
}

impl MinecraftToGamelord {
    pub fn minecraft_id(&self) -> &String {
        match self {
            MinecraftToGamelord::ValidateMove { minecraft_id, .. } => minecraft_id,
            MinecraftToGamelord::PlayerSpawnRequest { minecraft_id } => minecraft_id,
        }
    }

    pub fn cube(&self) -> Option<&Cube> {
        match self {
            MinecraftToGamelord::ValidateMove { cube, .. } => Some(cube),
            _ => None,
        }
    }
}


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
