import React, { useEffect, useState } from "react";
import { applyDiff } from "./shared";

function App() {
  const [lobby, setLobby] = useState({
    name: "",
    minecraft_server_address: "",
    team1: {
      last_message_id: 0,
      messages: [],
      name: "",
      players: [],
    },
    team2: {
      last_message_id: 0,
      messages: [],
      name: "",
      players: [],
    },
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
    // jurij's dev setup
    // 5173 - 8080
    // 5174 - 8080
    // 5175 - 8081
    const host =
      window.location.port === "5173" || window.location.port === "5174"
        ? "localhost:8080"
        : window.location.port === "5175"
        ? "localhost:8081"
        : window.location.host;
    
    const ws = new WebSocket(
      `${protocol}//${host}/mcclient:mcclient:basilesex.os/`
    );

    ws.onopen = function (event) {
      console.log("Connection opened on " + window.location.host + ":", event);
    };
    ws.onmessage = function (event) {
      const data = JSON.parse(event.data);
      console.log("data", data);
      applyDiff(data, setLobby);
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
        <p>
          <strong>Server Name:</strong> {lobby.name}
        </p>
        <p>
          <strong>Minecraft Server Address:</strong>{" "}
          {lobby.minecraft_server_address}
        </p>
        <h3>Teams</h3>
        <div>
          <h4>Team 1</h4>
          <ul>
            {lobby.team1?.players?.length > 0 ? (
              lobby.team1.players.map((player, index) => (
                <li key={index}>{player.kinode_id}</li>
              ))
            ) : (
              <li>No players in Team 1</li>
            )}
          </ul>
        </div>
        <div>
          <h4>Team 2</h4>
          <ul>
            {lobby.team2?.players?.length > 0 ? (
              lobby.team2.players.map((player, index) => (
                <li key={index}>{player.kinode_id}</li>
              ))
            ) : (
              <li>No players in Team 2</li>
            )}
          </ul>
        </div>
      </div>
      <pre id="response-output"></pre>
    </div>
  );
}

export default App;
