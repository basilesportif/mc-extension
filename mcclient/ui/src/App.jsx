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
import Msg from "./components/Msg";

function App({ws, ourNode, setOurNode, ourInTeam, setOurInTeam, lobby, setLobby, isReady, setIsReady}) {

  document.addEventListener("DOMContentLoaded", () => {
    document.getElementById("playerForm");
  });

  useEffect(() => {
    console.log("lobby", lobby);
    if (ourNode && lobby) {
      setOurInTeam(nodeInTeam(ourNode, lobby));
    }
  }, [lobby]);

  useEffect(() => {
    console.log("ourNode", ourNode);
    console.log("ourInTeam", ourInTeam);
  }, [ourNode, ourInTeam]);

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

  async function readyPlayer() {
    console.log("ready player");
    const url = `/mcclient:mcclient:basilesex.os/ready_player`;

    try {
      const response = await fetch(url, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(ourNode),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.text();
      document.getElementById(
        "response-output"
      ).innerText = `Ready player request made.`;
      setIsReady(true); // Set isReady to true
    } catch (error) {
      console.error("Error readying player:", error);
      document.getElementById(
        "response-output"
      ).innerText = `Error: ${error.message}`;
    }
  }

  const onSend = (message) => {
    console.log("sending:", message);
    ws.send(JSON.stringify({ SendMessage: message }));
  };

  return (
    <div className="ClientBackground">
      
      <h2>McClient</h2>
      {ourInTeam === null && (
        <>
          <h3>Join Team</h3>
          <div className="form-container">
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
              <div className="button-container">
                <button type="button" onClick={() => joinTeam("Team1")}>
                  Join Team1
                </button>
                <button type="button" onClick={() => joinTeam("Team2")}>
                  Join Team2
                </button>
              </div>
            </form>
          </div>
        </>
      )}
      <div className="status">
        <p>Game: {lobby.name}</p>
        <p>Minecraft Server Address: {lobby.minecraft_server_address}</p>
        <div className="team-container">
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
      </div>
      <pre id="response-output"></pre>
      {ourInTeam !== null && (
        <button type="button" onClick={readyPlayer}>
          Ready Player
        </button>
      )}
      <div
        style={{
          position: "relative",
        }}
      >
        {ourInTeam === null ? (
          <p>You are not in a team yet. Join a team to see messages.</p>
        ) : (
          <>
            <p>{ourInTeam} Chat</p>

            <MainContainer
              style={{
                width: "100%",
                height: "50vh",
                border: "1px solid #ccc",
                backgroundColor: "rgba(255, 255, 255, 0.4)", // Added background color
              }}
            >
              <ChatContainer>
                <MessageList
                  style={{
                    height: "50vh",
                    overflowY: "auto",
                    display: "flex",
                    flexDirection: "column-reverse",
                    border: "1px solid #ccc",
                  }}
                >
                  {nodeInTeam(ourNode, lobby) === "team1" ? (
                    lobby.team1.messages.map((message, index) => (
                      <Msg key={message.id} message={message} />
                    ))
                  ) : nodeInTeam(ourNode, lobby) === "team2" ? (
                    lobby.team2.messages.map((message, index) => (
                      <Msg key={message.id} message={message} />
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
                  style={{
                    border: "1px solid #ccc",
                    backgroundColor: "rgba(255, 255, 255, 0.4)", // Added background color
                  }}
                  placeholder="Type message here"
                  attachButton={false}
                  sendButton={false}
                  onSend={onSend}
                />
              </ChatContainer>
            </MainContainer>
          </>
        )}
      </div>
    </div>

  );
}

export default App;