import React, { useEffect, useState, useRef } from "react";
import { applyDiff } from "./shared";
import App from "./App";
import ThreeJsScene from "./MCWorld";
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

  const webSocket = () => {
    const protocol = window.location.protocol === "https:" ? "wss://" : "ws://";
    const host = window.location.host;

    ws = new WebSocket(`${protocol}//${host}/mcclient:mcclient:basilesex.os/`);

    ws.onopen = function (event) {
      console.log("Connection opened on " + window.location.host + ":", event);
    };
    ws.onmessage = function (event) {
      const data = JSON.parse(event.data);
      console.log("data", data);
      console.log("data in loby has been updated");
      applyDiff(data, setLobby);
      if (data.OurNode) {
        setOurNode(data.OurNode);
      }
    };
  };

  return (
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
  );
};

// Create a new component for the overlay
const OverlayComponent = () => {
  return (
    <div className="overlay">
      <h1>Game in progress...</h1>
    </div>
  );
};

export default InterfaceUI;