// warning: edit this file in mcstructs, not mcclient or gamelord
//
// note: doesn't live update with `npm run dev`
//
// here goes logic for applying diffs to lobby/state
export function applyDiff(data, setLobby) {
  console.log("KEY", Object.keys(data)[0]);
  switch (Object.keys(data)[0]) {
    case "EditLobby":
      console.log("Lobby edited:", data.EditLobby);
      setLobby((prevLobby) => ({
        ...prevLobby,
        name: data.EditLobby.name,
        minecraft_server_address: data.EditLobby.minecraft_server_address,
      }));
      break;
    case "AddPlayerToTeam":
      console.log("Player added to team:", data.AddPlayerToTeam);
      if (data.AddPlayerToTeam.team === "Team1") {
        setLobby((prevLobby) => ({
          ...prevLobby,
          team1: prevLobby.team1?.players
            ? {
                ...prevLobby.team1,
                players: [
                  ...prevLobby.team1.players,
                  data.AddPlayerToTeam.player,
                ],
              }
            : { ...prevLobby.team1, players: [data.AddPlayerToTeam.player] },
        }));
      } else {
        setLobby((prevLobby) => ({
          ...prevLobby,
          team2: prevLobby.team2?.players
            ? {
                ...prevLobby.team2,
                players: [
                  ...prevLobby.team2.players,
                  data.AddPlayerToTeam.player,
                ],
              }
            : { ...prevLobby.team2, players: [data.AddPlayerToTeam.player] },
        }));
      }
      break;
    case "Init":
      console.log("INIT", data.Init);
      setLobby(data.Init);
      break;
    case "Message":
      console.log("Message received:", data.Message);
      setLobby((prevLobby) => {
        let team = null;
        if (
          prevLobby.team1.players.some(
            (player) => player.kinode_id === data.Message.from.kinode_id
          )
        ) {
          team = "team1";
        } else if (
          prevLobby.team2.players.some(
            (player) => player.kinode_id === data.Message.from.kinode_id
          )
        ) {
          team = "team2";
        } else {
          console.error("Message from unknown player:", data.Message);
          return prevLobby;
        }
        return {
          ...prevLobby,
          [team]: {
            ...prevLobby[team],
            messages: [...prevLobby[team].messages, data.Message],
            last_message_id: data.Message.id,
          },
        };
      });
      console.log("Message", data.Message);
      break;
    case "ConfigurePoints":
      console.log("ConfigurePoints", data.ConfigurePoints);
      setLobby((prevLobby) => ({
        ...prevLobby,
        goal_post: data.ConfigurePoints.goal_post,
        team1: {
          ...prevLobby.team1,
          spawn_point: data.ConfigurePoints.team1_spawn,
        },
        team2: {
          ...prevLobby.team2,
          spawn_point: data.ConfigurePoints.team2_spawn,
        },
      }));
      break;
    case "WorldConfigRegion":
      console.log("WorldConfigRegion", data.WorldConfigRegion);
      let team = data.WorldConfigRegion[0] === "Team1" ? "team1" : "team2";
      setLobby((prevLobby) => ({
        ...prevLobby,
        world_config: {
          ...prevLobby.world_config,
          [team]: data.WorldConfigRegion[1],
        },
      }));
      break;
    default:
      console.log("Unknown websocket message:", data);
      break;
  }
}
