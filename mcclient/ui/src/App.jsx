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

function App({ws, ourNode, setOurNode, ourInTeam, setOurInTeam, lobby, setLobby, nodeInTeam}) {

  document.addEventListener("DOMContentLoaded", () => {
    document.getElementById("playerForm");
  });


  const onSend = (message) => {
    // console.log("sending:", message);
    ws.send(JSON.stringify({ SendMessage: message }));
  };

  return (
    <div>
      <h2>McClient</h2>
      <div style={{ textAlign: "left" }}>
        <p>Game: {lobby.name}</p>
        <p>Minecraft Server Address: {lobby.minecraft_server_address}</p>
        <div style={{ display: "flex", justifyContent: "space-between" }}>
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
                  style={{ border: "1px solid #ccc" }}
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
