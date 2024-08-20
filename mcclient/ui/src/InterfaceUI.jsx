import React, { useEffect, useState, useRef } from "react";
import { applyDiff } from "./shared";
import App from "./App";
import ThreeJsScene from "./MCWorld";
import Login from "./Login";
import './index.css'

let ws;

const InterfaceUI = () => {
  const [ourNode, setOurNode] = useState(null);
  const [ourInTeam, setOurInTeam] = useState(null);
  const [isReady, setIsReady] = useState(false); // Moved isReady state here
  const [lobby, setLobby] = useState({
    name: "",
    minecraft_server_address: "",
    world_config: {},
    goal_post: { center: [0, 0, 0], side_length: 16 },
    team1: {
      last_message_id: 0,
      messages: [],
      name: "",
      players: [],
      spawn_point: { center: [0, 0, 0], side_length: 16 },
    },
    team2: {
      last_message_id: 0,
      messages: [],
      name: "",
      players: [],
      spawn_point: { center: [0, 0, 0], side_length: 16 },
    },
    game_started: false,
    ready_players: [],
  });

  // Add a state to control the visibility of the overlay
  const [showOverlay, setShowOverlay] = useState(lobby.game_started);

  useEffect(() => {
    // Update the showOverlay state whenever lobby.game_started changes
    setShowOverlay(lobby.game_started);
  }, [lobby.game_started]);

  useEffect(() => {
    webSocket();
  }, []);

  useEffect(() => {
    // console.log("lobby", lobby);
    if (ourNode && lobby) {
      setOurInTeam(nodeInTeam(ourNode, lobby));
    }
  }, [lobby]);

  useEffect(() => {
     console.log("ourNode", ourNode);
     console.log("ourInTeam", ourInTeam);
  }, [ourNode, ourInTeam]);

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
      // console.log("data", data);
      applyDiff(data, setLobby);
      if (data.OurNode) {
        setOurNode(data.OurNode);
      }
    };
  };
  // Create a new component for the overlay
const OverlayComponent = () => {
  return (
    <div className="overlay">
      <h1>Game in progress...</h1>
    </div>
  );
};
// Possible bug with ourInTeam not being set/read correctly
// 
return (
  <div>
    {ourInTeam === null ? (
      <Login
        ourInTeam={ourInTeam}
        setOurInTeam={setOurInTeam}
        ourNode={ourNode}
        setOurNode={setOurNode}
      />
    ) : (
      <div style={{ display: "flex", height: "100vh" }}>
        <div  
          style={{
            width: "30%",
            padding: "20px",
            borderRight: "1px solid #ccc",
            overflowY: "auto",
          }}
        >
          <div className="container">
            {showOverlay ? (
              <OverlayComponent />
            ) : (
              <>
                <div className="left-panel">
                  <App
                    ws={ws}
                    ourNode={ourNode}
                    setOurNode={setOurNode}
                    ourInTeam={ourInTeam}
                    setOurInTeam={setOurInTeam}
                    lobby={lobby}
                    setLobby={setLobby}
                    isReady={isReady} // Pass isReady as a prop
                    setIsReady={setIsReady} // Pass setIsReady as a prop
                    nodeInTeam={nodeInTeam}
                  />
                </div>
                <div className="right-panel">
                  <ThreeJsScene
                    ws={ws}
                    ourInTeam={ourInTeam}
                    lobby={lobby}
                  />
                  {isReady && (
                    <div className="overlay">
                      <p>Waiting for other players...</p>
                    </div>
                  )}
                </div>
              </>
            )}
          </div>
        </div>
      </div>
    )}
  </div>
);
}
export default InterfaceUI;
