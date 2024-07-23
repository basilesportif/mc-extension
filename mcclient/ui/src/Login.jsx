import React, { useEffect, useState } from "react";
import { applyDiff } from "./shared";
import App from "./App";

const Login = ({ ourInTeam, setOurInTeam }) => {
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
    } catch (error) {
      console.error("Error adding player:", error);
    }
  }

  return (
    <div>
      <h1>Login</h1>
      {ourInTeam === null && (
        <>
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
        </>
      )}
    </div>
  );
};

export default Login;
