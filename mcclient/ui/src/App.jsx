import React, { useEffect, useState } from "react";
import { applyDiff } from "./shared";
import "./App.css"; // use for styling the chat
import {
  MainContainer,
  ChatContainer,
  MessageList,
  Message,
  MessageInput,
} from "@chatscope/chat-ui-kit-react";
let ws;

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
  const [ourNode, setOurNode] = useState(null);

  document.addEventListener("DOMContentLoaded", () => {
    document.getElementById("playerForm");
  });

  useEffect(() => {
    webSocket();
  }, []);

  useEffect(() => {
    console.log("lobby", lobby);
  }, [lobby]);

  useEffect(() => {
    console.log("ourNode", ourNode);
  }, [ourNode]);

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

  const nodeInTeam = (node, lobby) => {
    if (lobby.team1.players.some((p) => p.kinode_id === node)) {
      return "team1";
    } else if (lobby.team2.players.some((p) => p.kinode_id === node)) {
      return "team2";
    } else {
      return null;
    }
  };

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

    ws = new WebSocket(`${protocol}//${host}/mcclient:mcclient:basilesex.os/`);

    ws.onopen = function (event) {
      console.log("Connection opened on " + window.location.host + ":", event);
    };
    ws.onmessage = function (event) {
      const data = JSON.parse(event.data);
      console.log("data", data);
      applyDiff(data, setLobby);
      if (data.OurNode) {
        setOurNode(data.OurNode);
      }
    };
  };

  const onSend = (message) => {
    console.log("sending:", message);
    ws.send(JSON.stringify({ SendMessage: message }));
  };

  return (
    <div>
      <h2>McClient</h2>
      <h3>Join Team</h3>
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
        <p>Server Name: {lobby.name}</p>
        <p>Minecraft Server Address: {lobby.minecraft_server_address}</p>
        <div>
          <h4>Team 1</h4>
          <ul>
            {lobby.team1?.players?.length > 0 ? (
              lobby.team1.players.map((player, index) => (
                <p key={index}>{player.kinode_id}</p>
              ))
            ) : (
              <p>No players in Team 1</p>
            )}
          </ul>
        </div>
        <div>
          <h4>Team 2</h4>
          <ul>
            {lobby.team2?.players?.length > 0 ? (
              lobby.team2.players.map((player, index) => (
                <p key={index}>{player.kinode_id}</p>
              ))
            ) : (
              <p>No players in Team 2</p>
            )}
          </ul>
        </div>
      </div>
      <pre id="response-output"></pre>
      <div
        style={{
          position: "relative",
          marginLeft: "20px",
          marginRight: "20px",
        }}
      >
        <MainContainer style={{ width: "100%" }}>
          <ChatContainer>
            <MessageList>
              {nodeInTeam(ourNode, lobby) === "team1" ? (
                lobby.team1.messages.map((message, index) => (
                  <Message key={message.id} model={{ message: message.msg }}>
                    <Message.Header
                      sender={message.from.kinode_id}
                      sentTime={new Date(message.time * 1000).toLocaleString()}
                    />
                  </Message>
                ))
              ) : nodeInTeam(ourNode, lobby) === "team2" ? (
                lobby.team2.messages.map((message, index) => (
                  <Message 
                    key={message.id}
                    model={{ message: message.msg }}
                    sender={message.from.kinode_id}
                    sentTime={new Date(message.time * 1000).toLocaleString()}
                  />
                ))
              ) : (
                <Message
                  model={{
                    message:
                      "You are not in a team yet. Join a team to see messages.",
                    sentTime: "",
                    sender: "System",
                  }}
                />
              )}
            </MessageList>
            <MessageInput
              placeholder="Type message here"
              attachButton={false}
              onSend={onSend}
            />
          </ChatContainer>
        </MainContainer>
      </div>
    </div>
  );
}

export default App;
