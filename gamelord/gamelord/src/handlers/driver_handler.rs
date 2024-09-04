use kinode_process_lib::http::{send_ws_push, WsMessageType};
use kinode_process_lib::{
    println, Address, LazyLoadBlob,
    Message, Response, 
};

use crate::{
    ActivePlayer, GamelordRequestMinecraft, GamelordResponseMinecraft,
    State, TeamName, GamelordCaller, GameLobbyDiff
};

pub fn handle_driver_message(
    state: &mut State,
    gamelord_caller: &mut Option<GamelordCaller>,
    ws_channel_id: &mut Option<u32>,
    message: &Message,
    our: &Address,
) -> anyhow::Result<()> {
    //TODO: make sure the source of the message is mcclient
    println!("handle kinode message entered");
    match GamelordRequestMinecraft::parse(message.body())? {
        GamelordRequestMinecraft::CubeTransitionRequest { minecraft_id, cube } => {
            //check whether the playing phase has started ()
            if !state.lobby.game_started {
                println!("Action denied, playing phase has not started.");

                // Send the response to the player
                let response = serde_json::to_vec(
                    &GamelordResponseMinecraft::TransitionDeniedResponse(
                        "Action denied".to_string(),
                    ),
                ).expect("failed to parse gamelord transition denied response");
                Response::new().body(response).send().unwrap();
                return Ok(());
            }
            // check if the game is over
            if cube == state.lobby.goal_post {
                println!("Goal post reached, game over!");

                // Determine the winning team
                let winning_team = if let Some(active_player) = state.active_players.get(&minecraft_id) {
                    active_player.team.clone()
                } else {
                    return Err(anyhow::anyhow!("Player not found in active players"));
                };

                state.clear_teams(); // clear state on gamelord
                // Clear the world config
                state.lobby.clear_teams(); // clear_lobby

                // Send the GameOver diff to clients, including the cleared world_config
                let diff = GameLobbyDiff::GameOver {
                    game_started: false,
                };
                let _ = state.update_clients(&diff); //send the clear lobby diff to clients
                // Save the updated state
                state.save();
                 // Fetch team players and their spawn points
                let team1_players = state.lobby.team1.players.clone();
                let team2_players = state.lobby.team2.players.clone();
                let team1_spawn = state.lobby.team1.spawn_point.clone();
                let team2_spawn = state.lobby.team2.spawn_point.clone();
                state.save();
                // Send the response to the player
                let response = serde_json::to_vec(
                    &GamelordResponseMinecraft::GameOver {
                        winning_team: winning_team.clone(),
                        team1_players,
                        team2_players,
                        team1_spawn,
                        team2_spawn,
                    }
                ).expect("failed to parse gamelord cube transition response");
                let message = serde_json::to_vec(&serde_json::json!({ "winning_team": winning_team }))
                    .expect("failed to serialize JSON");
                let blob = LazyLoadBlob {
                    mime: Some("application/json".to_string()),
                    bytes: message,
                };
                send_ws_push(ws_channel_id.unwrap_or(0), WsMessageType::Text, blob);
                Response::new().body(response).send().unwrap();
                
                println!("winning team: {:?}", winning_team.clone());
                return Ok(());
            }
            // we get information about what team the player is in based on Active players
            if let Some(active_player) = state.active_players.get_mut(&minecraft_id) {
                println!(
                    "Player {} is active in the game on team {:?}.",
                    minecraft_id, active_player.team
                );

                // Update the player's current cube
                active_player.current_cube = cube.clone();

                let world_config = &state.lobby.world_config;
                let cube_to_owner = &state.cube_to_owner;

                if let Some(owners) = cube_to_owner.get(&cube) {
                    if !owners.contains(&active_player.team) {
                        // Cube is owned by enemy team(s)
                        for owner in owners {
                            if let Some(team_cubes) = world_config.get(owner) {
                                if let Some(cube_effects) = team_cubes.to_hashmap().get(&cube) {
                                    println!(
                                        "Cube effects for enemy owner {:?}: {:?}",
                                        owner, cube_effects
                                    );
                                    // Send cube effects to mcdriver
                                    let response = serde_json::to_vec(
                                        &GamelordResponseMinecraft::TransitionTriggeredResponse(
                                            minecraft_id.clone(),
                                            cube_effects.clone(),
                                        ),
                                    )
                                    .expect("failed to parse gamelord cube transition response");
                                    Response::new().body(response).send().unwrap();
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
                // If we reach here, either the cube is not owned, or it's owned by the player's team
                println!("Cube not in enemy region, you are clear");
                let response =
                        serde_json::to_vec(&GamelordResponseMinecraft::TransitionSilentResponse(
                            minecraft_id.clone(),
                            "No effects applied".to_string(),
                        ))
                            .expect("failed to parse gamelord cube transition response");
                Response::new().body(response).send().unwrap();
                Ok(())
            } else {
                println!("Player {} is not active in the game.", minecraft_id);
                Err(anyhow::anyhow!("Player not found in active players"))
            }
        }

        // this comes from MC-Driver
        // TO DO, connect this to the team registration interface for checking
        GamelordRequestMinecraft::PlayerSpawnRequest { minecraft_id } => {
            println!(
                "Player spawn request received for player: {:?}",
                minecraft_id
            );

            let team_name = if state
                .lobby
                .team1
                .players
                .iter()
                .any(|p| p.minecraft_player_name == minecraft_id)
            {
                TeamName::Team1
            } else if state
                .lobby
                .team2
                .players
                .iter()
                .any(|p| p.minecraft_player_name == minecraft_id)
            {
                TeamName::Team2
            } else {
                println!("Player {} is not assigned to a team", minecraft_id);
                let response =
                    serde_json::to_vec(&GamelordResponseMinecraft::PlayerSpawnRequestDenied(
                        false,
                        "Player is not assigned to a team.".to_string(),
                    ))
                    .expect("Failed to serialize response");
                Response::new().body(response).send().unwrap();
                return Ok(());
            };

            let spawn_cube = match team_name {
                TeamName::Team1 => &state.lobby.team1.spawn_point,
                TeamName::Team2 => &state.lobby.team2.spawn_point,
            };

            let active_player = ActivePlayer {
                kinode_id: minecraft_id.clone(),
                minecraft_player_name: minecraft_id.clone(),
                current_cube: spawn_cube.clone(),
                team: team_name.clone(),
            };

            state
                .active_players
                .insert(minecraft_id.clone(), active_player);

            println!(
                "Player {} added to active players on team {:?}",
                minecraft_id, &team_name
            );

            let response =
                serde_json::to_vec(&GamelordResponseMinecraft::PlayerSpawnRequestAuthorized(
                    true,
                    format!("Player added to team {:?}.", &team_name),
                    spawn_cube.clone(),
                ))
                .expect("Failed to serialize response");
            Response::new().body(response).send().unwrap();
            Ok(())
        }
        // have the option that someone can leave the game
        GamelordRequestMinecraft::PlayerLeaveRequest { minecraft_id } => {
            if state.active_players.contains_key(&minecraft_id) {
                state.active_players.remove(&minecraft_id);
                println!("Player with kinode_id {} has left the game.", &minecraft_id);
                state.save();
            } else {
                println!(
                    "Player with kinode_id {} is not in the active players list.",
                    &minecraft_id
                );
            }
            Ok(())
        }
    }
}