import React, { useEffect, useState } from "react";
import { applyDiff } from "./shared";
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
  const [activeTab, setActiveTab] = useState("tab1");
  const [wsReady, setWsReady] = useState(false);

  useEffect(() => {
    document.getElementById("playerForm").addEventListener("submit", addPlayer);
    setActiveTab("tab2");
    if (!wsReady) {
      webSocket();
    }
  }, []);

  useEffect(() => {
    console.log("LOBBY:", lobby);
  }, [lobby]);

  useEffect(() => {
    // Remove 'active' class from all tabs
    document.querySelectorAll(".tab-content").forEach((tab) => {
      tab.classList.remove("active");
    });
    // Add 'active' class to the selected tab
    document.getElementById(activeTab).classList.add("active");
  }, [activeTab]);

  async function addPlayer() {
    const minecraftName = document.getElementById("minecraftName").placeholder;
    const kinode_id = document.getElementById("kinode_id").placeholder;
    const playerData = {
      kinode_id: kinode_id,
      minecraft_player_name: minecraftName,
    };

    const url = "/gamelord:gamelord:basilesex.os/api/addPlayer"; // Adjust the URL as needed

    try {
      const response = await fetch(url, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(playerData),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.text();
      console.log("Player added successfully:", data);
      document.getElementById(
        "response-output-tab1"
      ).innerText = `Player ${minecraftName} with ID ${kinode_id} added successfully.`;
    } catch (error) {
      console.error("Error adding player:", error);
      document.getElementById(
        "response-output-tab1"
      ).innerText = `Error: ${error.message}`;
    }
  }

  function uploadFile() {
    const fileInput = document.getElementById("fileInput");
    const file = fileInput.files[0];
    if (file) {
      const reader = new FileReader();
      reader.onload = async (e) => {
        const text = e.target.result;
        try {
          const jsonData = JSON.parse(text);
          const url = "/gamelord:gamelord:basilesex.os/api/loadWorld";
          const response = await fetch(url, {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify(jsonData),
          });

          if (response.ok) {
            console.log("JSON uploaded successfully.");
            document.getElementById("response-output-tab1").innerText =
              "JSON uploaded successfully.";
          } else {
            throw new Error("Failed to upload JSON");
          }
        } catch (error) {
          console.error("Error reading or sending file:", error);
          document.getElementById(
            "response-output-tab1"
          ).innerText = `Error: ${error.message}`;
        }
      };
      reader.readAsText(file);
    } else {
      console.log("No file selected.");
      document.getElementById("response-output-tab1").innerText =
        "No file selected.";
    }
  }

  function deleteWorld() {
    const url = "/gamelord:gamelord:basilesex.os/api/deleteWorld"; // Adjust the URL as needed

    fetch(url, {
      method: "POST",
    })
      .then((response) => {
        if (response.ok) {
          console.log("World deleted successfully.");
          document.getElementById("response-output-tab1").innerText =
            "World deleted successfully.";
        } else {
          throw new Error("Failed to delete world");
        }
      })
      .catch((error) => {
        console.error("Error deleting world:", error);
        document.getElementById(
          "response-output-tab1"
        ).innerText = `Error: ${error.message}`;
      });
  }

  function getWorldConfig() {
    const url = "/gamelord:gamelord:basilesex.os/world_config";

    fetch(url, {
      method: "GET",
    })
      .then((response) => {
        if (response.ok) {
          return response.json();
        } else {
          throw new Error("Failed to get world configuration");
        }
      })
      .then((data) => {
        console.log("World configuration:", data);
        document.getElementById("response-output-tab1").innerText =
          JSON.stringify(data, null, 2);
      })
      .catch((error) => {
        console.error("Error getting world configuration:", error);
        document.getElementById(
          "response-output-tab1"
        ).innerText = `Error: ${error.message}`;
      });
  }

  const clearTeams = () => {
    const url = "/gamelord:gamelord:basilesex.os/api/clearTeams";

    fetch(url, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
    })
      .then((response) => {
        if (response.ok) {
          return response.text();
        } else {
          throw new Error("Failed to edit lobby");
        }
      })
      .then((result) => {
        document.getElementById("response-output-tab2").innerText = result;
      })
      .catch((error) => {
        console.error("Error editing lobby:", error);
        document.getElementById(
          "response-output-tab2"
        ).innerText = `Error: ${error.message}`;
      });
  };

  const editLobby = () => {
    const lobbyName = document.getElementById("lobbyName").value || lobby.name;
    const minecraftServerAddress =
      document.getElementById("minecraftServerAddress").value ||
      lobby.minecraft_server_address;

    const url = "/gamelord:gamelord:basilesex.os/api/editLobby";

    const data = {
      EditLobby: [lobbyName, minecraftServerAddress],
    };
    // console.log(data);

    fetch(url, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(data),
    })
      .then((response) => {
        if (response.ok) {
          return response.text();
        } else {
          throw new Error("Failed to edit lobby");
        }
      })
      .then((result) => {
        // console.log("Lobby edited successfully:", result);
        document.getElementById("response-output-tab2").innerText = result;
      })
      .catch((error) => {
        console.error("Error editing lobby:", error);
        document.getElementById(
          "response-output-tab2"
        ).innerText = `Error: ${error.message}`;
      });
  };

  const webSocket = () => {
    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const host =
      window.location.port === "5173" ? "localhost:8080" : window.location.host;
    if (!wsReady) {
      ws = new WebSocket(`${protocol}//${host}/gamelord:gamelord:basilesex.os/`);
    }

    ws.onopen = function (event) {
      console.log("Connection opened on " + window.location.host + ":", event);
      setWsReady(true);
      // ws.send(JSON.stringify("GetInit"));
    };
    ws.onmessage = function (event) {
      const data = JSON.parse(event.data);
      applyDiff(data, setLobby);
    };
  };

  return (
    <div>
      <h1>Gamelord</h1>
      <div className="tabs">
        <div className="tab" onClick={() => setActiveTab("tab1")}>
          Gamelord
        </div>
        <div className="tab" onClick={() => setActiveTab("tab2")}>
          Lobby Manager
        </div>
      </div>
      <div id="tab1" className="tab-content active">
        <div className="container">
          <div className="option">
            <h2>Add Player to Game</h2>
            <form id="playerForm">
              <div className="form-group">
                <input
                  type="text"
                  id="minecraftName"
                  name="minecraftName"
                  placeholder="Enter Minecraft Name"
                />
              </div>
              <div className="form-group">
                <input
                  type="text"
                  id="kinode_id"
                  name="kinode_id"
                  placeholder="Enter Kinode ID"
                />
              </div>
              <button type="button" onClick={() => addPlayer()}>
                Submit Details
              </button>
            </form>
          </div>
          <div className="option">
            <h2>Load Preconfigured World</h2>
            <form>
              <div className="form-group">
                <input type="file" id="fileInput" accept=".json" />
              </div>
              <button type="button" onClick={() => uploadFile()}>
                Upload File
              </button>
            </form>
          </div>
        </div>
        <div className="container">
          <div className="option">
            <h2>Delete World</h2>
            <button type="button" onClick={() => deleteWorld()}>
              Delete World
            </button>
          </div>
          <div className="option">
            <h2>Get World Configuration</h2>
            <button type="button" onClick={() => getWorldConfig()}>
              Get World Config
            </button>
          </div>
        </div>
        <pre id="response-output-tab1"></pre>
      </div>
      <div id="tab2" className="tab-content active">
        <div className="container">
          <div className="option">
            <h2>Edit Lobby</h2>
            <form id="editLobbyForm">
              <div className="form-group">
                <input
                  type="text"
                  id="lobbyName"
                  name="lobbyName"
                  placeholder={`Current Name: ${lobby.name}` || "Enter Lobby Name"}
                />
              </div>
              <div className="form-group">
                <input
                  type="text"
                  id="minecraftServerAddress"
                  name="minecraftServerAddress"
                  placeholder={`Current Server: ${lobby.minecraft_server_address}` || "Enter Minecraft Server Address"}
                />
              </div>
              <div className="form-group"></div>
              <button type="button" onClick={() => editLobby()}>
                Submit Changes
              </button>
            </form>
          </div>
          <div className="option">
            <h2>Team Members</h2>
            <div>
              <h3>Team 1</h3>
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
              <h3>Team 2</h3>
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
            <button type="button" onClick={() => clearTeams()}>
              Clear Teams
            </button>
          </div>
        </div>
        <pre id="response-output-tab2"></pre>
      </div>
    </div>
  );
}

export default App;
