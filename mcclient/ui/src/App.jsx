import React, { useEffect, useState } from "react";

function App() {
  const [lobby, setLobby] = useState({
    name: '',
    minecraft_server_address: '',
    team1: [],
    team2: []
  });

  document.addEventListener("DOMContentLoaded", () => {
    document.getElementById("playerForm");
  });

  useEffect(() => {
    webSocket();
  }, []);

  async function joinTeam(team_name) {
    console.log("join team");
    const minecraft_id = document.getElementById("minecraftId").value;
    const gamelord_id = document.getElementById("gamelordId").value;

    const url = `/mcclient:mcclient:basilesex.os/join_team`;

    try {
      const response = await fetch(url, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          gamelord_id: gamelord_id,
          minecraft_id: minecraft_id,
          team_name: team_name,
        }),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.text();
      document.getElementById(
        "response-output"
      ).innerText = `${minecraft_id} join request made.`;
    } catch (error) {
      console.error("Error adding player:", error);
      document.getElementById(
        "response-output"
      ).innerText = `Error: ${error.message}`;
    }
  }

  const webSocket = () => {
    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const host =
      window.location.port === "5173" ? "localhost:8080" : window.location.host;
    const ws = new WebSocket(
      `${protocol}//${host}/mcclient:mcclient:basilesex.os/`
    );

    ws.onopen = function (event) {
      console.log("Connection opened on " + window.location.host + ":", event);
    };
    ws.onmessage = function (event) {
      const data = JSON.parse(event.data);
      switch (Object.keys(data)[0]) {
        case 'EditLobby':
          console.log('Lobby edited:', data.EditLobby);
          setLobby(prevLobby => ({
            ...prevLobby,
            name: data.EditLobby.name,
            minecraft_server_address: data.EditLobby.minecraft_server_address,
          }));

        // case 'Init':
        //   console.log('Game lobby:', data.Init);
        //   setLobby(data.Init);
        default:
          console.log('Unknown websocket message:', data);
      }

    };
  };

  return (
    <div>
      <h1>McClient</h1>
      <h2>Join Team</h2>
      <form id="playerForm">
        <input
          type="text"
          id="gamelordId"
          placeholder="Enter Gamelord NodeId, e.g. gamelordd.os"
        />
        <input
          type="text"
          id="minecraftId"
          placeholder="Enter Your Minecraft ID"
        />
        <button type="button" onClick={() => joinTeam("Team1")}>
          Join Team1
        </button>
        <button type="button" onClick={() => joinTeam("Team2")}>
          Join Team2
        </button>
      </form>
      <div>
        <h2>Game Lobby Information</h2>
        <p><strong>Server Name:</strong> {lobby.name}</p>
        <p><strong>Minecraft Server Address:</strong> {lobby.minecraft_server_address}</p>
        <h3>Teams</h3>
        <div>
          <h4>Team 1</h4>
          <ul>
            {lobby.team1.map((player, index) => (
              <li key={index}>{player.minecraft_player_name}</li>
            )) || 'No players'}
          </ul>
        </div>
        <div>
          <h4>Team 2</h4>
          <ul>
            {lobby.team2.map((player, index) => (
              <li key={index}>{player.minecraft_player_name}</li>
            )) || 'No players'}
          </ul>
        </div>
      </div>
      <pre id="response-output"></pre>
    </div>
  );
}

export default App;
