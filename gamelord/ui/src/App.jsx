import React, { useEffect, useState } from "react";
import { applyDiff } from "./shared";
let ws;
function App() {
  const [lobby, setLobby] = useState({
    name: "",
    minecraft_server_address: "",
    world_config: {},
    goal_post: {center: [0,0,0], side_length: 16},
    team1: {
      last_message_id: 0,
      messages: [],
      name: "",
      players: [],
      spawn_point: {center: [0,0,0], side_length: 16},
    },
    team2: {
      last_message_id: 0,
      messages: [],
      name: "",
      players: [],
      spawn_point: {center: [0,0,0], side_length: 16},
    },
    game_started: false,
    ready_players: [],
  });
  const [activeTab, setActiveTab] = useState("tab1");
  const [wsReady, setWsReady] = useState(false);
  const [winningTeam, setWinningTeam] = useState(null);

  useEffect(() => {
    setActiveTab("tab2");
    if (!wsReady) {
      webSocket();
    }
  }, []);

  useEffect(() => {
    console.log("LOBBY:", lobby);
  }, [lobby]);

  // New effect to listen for changes in ready_players
  useEffect(() => {
    if (lobby.ready_players.length > 0) {
      console.log("New ready players:", lobby.ready_players);
      // You can add additional logic here to display the ready players
      // or perform any other necessary actions
    }
  }, [lobby.ready_players]);

  useEffect(() => {
    // Remove 'active' class from all tabs
    document.querySelectorAll(".tab-content").forEach((tab) => {
      tab.classList.remove("active");
    });
    // Add 'active' class to the selected tab
    document.getElementById(activeTab).classList.add("active");
  }, [activeTab]);

  useEffect(() => {
    // Ensure the event listener is set up correctly after the component mounts
    const uploadButton = document.getElementById("uploadFolderButton");
    if (uploadButton) {
      uploadButton.addEventListener("click", uploadFolder);
    }

    // Cleanup event listener on component unmount
    return () => {
      if (uploadButton) {
        uploadButton.removeEventListener("click", uploadFolder);
      }
    };
  }, []);

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

  async function uploadFolder() {
    const fileInput = document.getElementById("folderInput");
    const files = fileInput.files;
    console.log("Files:", files);

    if (files.length > 0) {
      const filesObject = {};
      const totalFiles = files.length;
      let uploadedFiles = 0;

      for (let i = 0; i < files.length; i++) {
        const file = files[i];
        const content = await readFileAsBase64(file);
        filesObject[file.webkitRelativePath] = content;
        console.log("File:", file.webkitRelativePath);

        // Update progress
        uploadedFiles++;
        const progress = Math.round((uploadedFiles / totalFiles) * 100);
        document.getElementById("upload-progress").innerText = `Progress: ${progress}%`;
      }
      
      const url = "/gamelord:gamelord:basilesex.os/api/loadFolder";
      console.log("Sending request to:", url);
      console.log("Files object:", filesObject);

      console.log("body:", JSON.stringify(filesObject));
      
      fetch(url, {
        method: "POST",
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(filesObject),
      })
        .then((response) => {
          if (response.ok) {
            return response.text();
          } else {
            throw new Error("Failed to upload folder");
          }
        })
        .then((result) => {
          console.log("Folder uploaded successfully.");
          document.getElementById("response-output-tab2").innerText = "Folder uploaded successfully.";
        })
        .catch((error) => {
          console.error("Error uploading folder:", error);
          document.getElementById("response-output-tab2").innerText = `Error: ${error.message}`;
        });
    } else {
      console.log("No folder selected.");
      document.getElementById("response-output-tab2").innerText = "No folder selected.";
    }
  }

  function readFileAsBase64(file) {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(reader.result.split(',')[1]);
      reader.onerror = error => reject(error);
      reader.readAsDataURL(file);
    });
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
        // Create a Blob with the JSON data
        const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
        
        // Create a temporary URL for the Blob
        const url = window.URL.createObjectURL(blob);
        
        // Create a temporary anchor element
        const a = document.createElement('a');
        a.href = url;
        a.download = 'world_config.json';
        
        // Append the anchor to the body, click it, and remove it
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        
        // Revoke the temporary URL
        window.URL.revokeObjectURL(url);
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

  const configurePoints = () => {
    const team1SpawnX = parseInt(document.getElementById("team1SpawnX").value || "0", 10);
    const team1SpawnY = parseInt(document.getElementById("team1SpawnY").value || "0", 10);
    const team1SpawnZ = parseInt(document.getElementById("team1SpawnZ").value || "0", 10);
    const team2SpawnX = parseInt(document.getElementById("team2SpawnX").value || "0", 10);
    const team2SpawnY = parseInt(document.getElementById("team2SpawnY").value || "0", 10);
    const team2SpawnZ = parseInt(document.getElementById("team2SpawnZ").value || "0", 10);
    const goalPostX = parseInt(document.getElementById("goalPostX").value || "0", 10);
    const goalPostY = parseInt(document.getElementById("goalPostY").value || "0", 10);
    const goalPostZ = parseInt(document.getElementById("goalPostZ").value || "0", 10);

    const data = {
      team1_spawn: { center: [team1SpawnX, team1SpawnY, team1SpawnZ], side_length: 16 },
      team2_spawn: { center: [team2SpawnX, team2SpawnY, team2SpawnZ], side_length: 16 },
      goal_post: { center: [goalPostX, goalPostY, goalPostZ], side_length: 16 },
    };

    console.log("configuring points:", data);
    ws.send(JSON.stringify({ ConfigurePoints: data }));
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
      ws = new WebSocket(
        `${protocol}//${host}/gamelord:gamelord:basilesex.os/`
      );
    }

    ws.onopen = function (event) {
      console.log("Connection opened on " + window.location.host + ":", event);
      setWsReady(true);
      // ws.send(JSON.stringify("GetInit"));
    };
    ws.onmessage = function (event) {
      const data = JSON.parse(event.data);
      console.log("Received WebSocket message:", data); // Debugging line

      // Check if the data contains a winning_team field
      if (data.winning_team) {
        console.log("Winning team received:", data.winning_team);
        if (data.winning_team === "Team1" || data.winning_team === "Team2") {
          setWinningTeam(data.winning_team);
        } else {
          console.warn("Unexpected winning_team value:", data.winning_team);
        }
      } else {
        console.warn("Unexpected WebSocket message:", data);
      }

      applyDiff(data, setLobby);
    };
  };

  const lockGame = () => {
    const url = "/gamelord:gamelord:basilesex.os/api/lockGame";

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
          throw new Error("Failed to lock game");
        }
      })
      .then((result) => {
        document.getElementById("response-output-tab2").innerText = result;
      })
      .catch((error) => {
        console.error("Error locking game:", error);
        document.getElementById("response-output-tab2").innerText = `Error: ${error.message}`;
      });
  };

  const disperseFunds = () => {
    const url = "/gamelord:gamelord:basilesex.os/api/disperseFunds";
    const data = { winning_team: winningTeam };

    fetch(url, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(data),
    })
      .then((response) => {
        if (response.ok) {
          console.log("Funds dispersed successfully.");
          document.getElementById("response-output-tab2").innerText =
            "Funds dispersed successfully.";
          // Clear the winning team
          setWinningTeam(null);
        } else {
          throw new Error("Failed to disperse funds");
        }
      })
      .catch((error) => {
        console.error("Error dispersing funds:", error);
        document.getElementById("response-output-tab2").innerText = `Error: ${error.message}`;
      });
  };

  const ReadyPlayersList = ({ lobby }) => {
    return (
      <div>
        <h3>Ready Players</h3>
        <ul>
          {lobby.ready_players.map((player, index) => (
            <li key={index}>{player}</li>
          ))}
        </ul>
      </div>
    );
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
            <h2>Load Preconfigured Effects</h2>
            <p>Insert world_config.json and overwrite current world_config. NOT necessary to use, players can start by editing an empty world config.</p>
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
            <p>Deletes the world config created by players.</p>
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
            <h2>Upload Folder</h2>
            <p>Upload a folder containing world configuration files.</p>
            <div className="form-group">
              <input type="file" id="folderInput" webkitdirectory="true" directory="true" multiple />
            </div>
            <button type="button" id="uploadFolderButton">
              Upload Folder
            </button>
            <div id="upload-progress"></div>
          </div>
          <div className="option">
            <h2>Edit Lobby</h2>
            <form id="editLobbyForm">
              <div className="form-group">
                <input
                  type="text"
                  id="lobbyName"
                  name="lobbyName"
                  placeholder={
                    `Current Name: ${lobby.name}` || "Enter Lobby Name"
                  }
                />
              </div>
              <div className="form-group">
                <input
                  type="text"
                  id="minecraftServerAddress"
                  name="minecraftServerAddress"
                  placeholder={
                    `Current Server: ${lobby.minecraft_server_address}` ||
                    "Enter Minecraft Server Address"
                  }
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
          <div className="option">
            <h2>Ready Players</h2>
            <ReadyPlayersList lobby={lobby} />
          </div>
          <div className="option">
            <h2>Configure Points</h2>
            <form id="configPointsForm">
              <div className="form-group">
                <input
                  type="number"
                  id="team1SpawnX"
                  name="team1SpawnX"
                  placeholder="Team 1 Spawn X"
                />
                <input
                  type="number"
                  id="team1SpawnY"
                  name="team1SpawnY"
                  placeholder="Team 1 Spawn Y"
                />
                <input
                  type="number"
                  id="team1SpawnZ"
                  name="team1SpawnZ"
                  placeholder="Team 1 Spawn Z"
                />
              </div>
              <div className="form-group">
                <input
                  type="number"
                  id="team2SpawnX"
                  name="team2SpawnX"
                  placeholder="Team 2 Spawn X"
                />
                <input
                  type="number"
                  id="team2SpawnY"
                  name="team2SpawnY"
                  placeholder="Team 2 Spawn Y"
                />
                <input
                  type="number"
                  id="team2SpawnZ"
                  name="team2SpawnZ"
                  placeholder="Team 2 Spawn Z"
                />
              </div>
              <div className="form-group">
                <input
                  type="number"
                  id="goalPostX"
                  name="goalPostX"
                  placeholder="Goal Post X"
                />
                <input
                  type="number"
                  id="goalPostY"
                  name="goalPostY"
                  placeholder="Goal Post Y"
                />
                <input
                  type="number"
                  id="goalPostZ"
                  name="goalPostZ"
                  placeholder="Goal Post Z"
                />
              </div>
              <div className="form-group"></div>
              <button type="button" onClick={() => configurePoints()}>
                Submit Changes
              </button>
            </form>
          </div>
          <div className="option">
            <h2>Begin Playing Phase</h2>
            <p>Begin the playing phase of the game.</p>
            <button type="button" onClick={() => lockGame()}>
              Begin Playing Phase
            </button>
          </div>
          <div className="option">
            <h2>Winning Team</h2>
            <p>{winningTeam ? `Winning Team: ${winningTeam}` : "No winning team yet."}</p>
            {winningTeam && (
              <button type="button" onClick={() => disperseFunds()}>
                Disperse Funds to Winning Team
              </button>
            )}
          </div>
        </div>
        <pre id="response-output-tab2"></pre>
      </div>
    </div>
  );
}

export default App;