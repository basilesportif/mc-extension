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
      console.log("Message", data.Message);
      break;
    default:
      console.log("Unknown websocket message:", data);
      break;
  }
}
