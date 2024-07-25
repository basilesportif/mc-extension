import React, { useEffect, useState } from "react";
import { ethers, parseEther } from "ethers";
import Gamelord from "./abi/Gamelord.json";

let signer = null;
let provider;
const CONTRACT_ADDRESS = "0x5fbdb2315678afecb367f032d93f642f64180aa3";
const Login = ({ ourInTeam, setOurInTeam }) => {
  const [userAccount, setUserAccount] = useState("");
  const [chainId, setChainId] = useState();
  const [contract, setContract] = useState(null);
  const [number, setNumber] = useState();
  const [ethWagered, setEthWagered] = useState();
  const [teamRequested, setTeamRequested] = useState();

  // allow button if eth wagered != 0
  async function joinTeamRequest(team_name) {
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

  async function register(team_name) {
    const eth_amount = document.getElementById("ethAmount").value;

    try {
      const team_name_as_num = team_name === "Team1" ? 0 : 1;
      const tx = await contract.wager(team_name_as_num, { value: parseEther(eth_amount) });
      const receipt = await tx.wait();

      console.log("TX", receipt);
      getPlayerInfo();
    } catch (error) {
      console.error(error);
    }
  }

  async function getPlayerInfo() {
    const tx = await contract.getPlayerInfo(userAccount);
    const player_info = await tx.wait();
    // setEthWagered(player_info.amount_wagered);
    // setTeamRequested(player_info.team);
    console.log("ETH WAGERED", player_info);
    // console.log("TEAM REQUESTED", player_info.team);
  }

  useEffect(() => {
    const loadEthers = async () => {
      if (window.ethereum == null) {
        console.log("MetaMask not installed; using read-only defaults");
        provider = ethers.getDefaultProvider();
      } else {
        provider = new ethers.BrowserProvider(window.ethereum);
        await provider.send("eth_requestAccounts", []);
        signer = await provider.getSigner();
        // Get the provider's chain ID
        let network = await provider.getNetwork();
        console.log("NETWORK", network.chainId);
        setChainId(network.chainId.toString());
        console.log("Connected to chain ID:", network.chainId);
        let address = await signer.getAddress();
        setUserAccount(address);
        console.log("ADDRESS", address);
        const gamelord_contract = new ethers.Contract(
          CONTRACT_ADDRESS,
          Gamelord.abi,
          signer
        );
        setContract(gamelord_contract);
      }
    };
    loadEthers();
  }, []);

  return (
    <>
      <h1>Login</h1>
      {ourInTeam === null && (
        <>
          <h3>Join Team</h3>
          <form id="playerForm">
            <input
              type="text"
              id="ethAmount"
              placeholder="Eth Amount to Wager (Ether)"
            />
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
            <button type="button" onClick={() => register("Team1")}>
              Join Team1
            </button>
            <button type="button" onClick={() => register("Team2")}>
              Join Team2
            </button>
          </form>
        </>
      )}
      <hr />
      <div>Chain ID: {chainId}</div>
      <div>Address: {userAccount}</div>
    </>
  );
};

export default Login;
