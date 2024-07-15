import React, { useEffect, useState } from "react";

function App() {
  const [lobby, setLobby] = useState({
    name: "",
    minecraft_server_address: "",
    team1: [],
    team2: [],
  });
  const [activeTab, setActiveTab] = useState("tab1");

  useEffect(() => {
    document.getElementById("playerForm").addEventListener("submit", addPlayer);
    getLobby();
    setActiveTab("tab1");
  }, []);

  useEffect(() => {
    // Remove 'active' class from all tabs
    document.querySelectorAll('.tab-content').forEach(tab => {
      tab.classList.remove('active');
    });
    // Add 'active' class to the selected tab
    document.getElementById(activeTab).classList.add('active');
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

  const editLobby = () => {
    const lobbyName = document.getElementById("lobbyName").value || lobby.name  ;
    const minecraftServerAddress = document.getElementById(
      "minecraftServerAddress"
    ).value || lobby.minecraft_server_address;
    const clearTeams = document.getElementById("clearTeams").checked;

    const url = "/gamelord:gamelord:basilesex.os/api/editLobby";

    const data = {
      name: lobbyName,
      minecraft_server_address: minecraftServerAddress,
      clear_teams: clearTeams,
    };
    console.log(data);

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
        console.log("Lobby edited successfully:", result);
        document.getElementById("response-output-tab2").innerText = result;
      })
      .catch((error) => {
        console.error("Error editing lobby:", error);
        document.getElementById(
          "response-output-tab2"
        ).innerText = `Error: ${error.message}`;
      });
  }

  function getLobby() {
    const url = "/gamelord:gamelord:basilesex.os/lobby";

    fetch(url)
      .then((response) => {
        if (response.ok) {
          return response.json();
        } else {
          throw new Error("Failed to get lobby data");
        }
      })
      .then((data) => {
        console.log("Lobby data retrieved successfully:", data);

        // Update the lobby information in the UI
        document.getElementById("lobbyName").placeholder =
          "Game Name: " + data.name;
        document.getElementById("minecraftServerAddress").placeholder =
          "MC Server: " + data.minecraft_server_address;

        setLobby({
          name: data.name,
          minecraft_server_address: data.minecraft_server_address,
          team1: data.team1.players,
          team2: data.team2.players,
        });

        document.getElementById("response-output-tab2").innerText =
          "Lobby data updated successfully";
      })
      .catch((error) => {
        console.error("Error getting lobby data:", error);
        document.getElementById(
          "response-output-tab2"
        ).innerText = `Error: ${error.message}`;
      });
  }

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
                  placeholder="Enter Lobby Name"
                />
              </div>
              <div className="form-group">
                <input
                  type="text"
                  id="minecraftServerAddress"
                  name="minecraftServerAddress"
                  placeholder="Enter Minecraft Server Address"
                />
              </div>
              <div className="form-group"></div>
              <div className="form-group">
                <div className="form-group">
                  <label htmlFor="clearTeams">
                    <input type="checkbox" id="clearTeams" name="clearTeams" />
                    Clear Teams
                  </label>
                </div>
              </div>
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
                {lobby.team1.map((player, index) => (
                  <li key={index}>{player.kinode_id}</li>
                ))}
              </ul>
            </div>
            <div>
              <h3>Team 2</h3>
              <ul>
                {lobby.team2.map((player, index) => (
                  <li key={index}>{player.kinode_id}</li>
                ))}
              </ul>
            </div>
          </div>
        </div>
        <pre id="response-output-tab2"></pre>
      </div>
    </div>
  );
}

export default App;
