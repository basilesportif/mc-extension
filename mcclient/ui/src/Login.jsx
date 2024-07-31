import React, { useEffect, useState } from "react";
import { ethers, parseEther } from "ethers";
import Gamelord from "./abi/Gamelord.json";

let signer = null;
let provider;

const CURRENT_CHAIN_ID = import.meta.env.VITE_CURRENT_CHAIN_ID;
let CONTRACT_ADDRESS;

switch (CURRENT_CHAIN_ID) {
  case "31337":
    CONTRACT_ADDRESS = import.meta.env.VITE_ANVIL_CONTRACT_ADDRESS;
    break;
  case "11155111":
    CONTRACT_ADDRESS = import.meta.env.VITE_SEPOLIA_CONTRACT_ADDRESS;
    break;
  case "1":
    CONTRACT_ADDRESS = import.meta.env.VITE_MAINNET_CONTRACT_ADDRESS;
    break;
  case "10":
    CONTRACT_ADDRESS = import.meta.env.VITE_OPTIMISM_CONTRACT_ADDRESS;
    break;
  default:
    throw new Error(`Invalid CURRENT_CHAIN_ID`);
}
console.log("CONTRACT_ADDRESS", CONTRACT_ADDRESS);

const Login = ({ ourInTeam, setOurInTeam, ourNode, setOurNode }) => {
  const [userAccount, setUserAccount] = useState("");
  const [chainId, setChainId] = useState();
  const [contract, setContract] = useState(null);
  const [number, setNumber] = useState();
  const [ethWagered, setEthWagered] = useState(null);
  const [teamRequested, setTeamRequested] = useState(null);

  // allow button if eth wagered != 0
  async function joinTeamRequest(team_name) {
    const minecraft_id = document.getElementById("minecraftId").value;
    const gamelord_id = document.getElementById("gamelordId").value;

    const url = `/mcclient:mcclient:basilesex.os/join_team`;

    const message = ourNode;
    const sig = await signer.signMessage(message);

    console.log(sig);

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
          eth_address: userAccount,
          signature: sig,
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
      console.log("TEAM NAME AS NUM", team_name_as_num);
      console.log("ETH AMOUNT", eth_amount);
      console.log("PARSED ETH AMOUNT", parseEther(eth_amount));
      const tx = await contract.wager(team_name_as_num, {
        value: parseEther(eth_amount),
      });
      console.log("TX", tx);
      const receipt = await tx.wait();
      // console.log("RECEIPT", receipt);
      getPlayerInfo();
    } catch (error) {
      console.error(error);
    }
  }

  async function getPlayerInfo() {
    const player_info = await contract.getPlayerInfo(userAccount);
    setEthWagered(player_info[0]);
    setTeamRequested(player_info[1] === 0n ? "Team1" : "Team2");
  }

  useEffect(() => {
    try {
      getPlayerInfo();
    } catch (error) {}
  }, [contract]);

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
        // console.log("NETWORK", network.chainId);
        setChainId(network.chainId.toString());
        // console.log("Connected to chain ID:", network.chainId);
        let address = await signer.getAddress();
        setUserAccount(address);
        // console.log("ADDRESS", address);
        const gamelord_contract = new ethers.Contract(
          CONTRACT_ADDRESS,
          Gamelord.abi,
          signer
        );
        setContract(gamelord_contract);
        setEthWagered(null);
        setTeamRequested(null);
      }
    };
    loadEthers();
  }, []);

  return (
    <>
      <h1>Login</h1>
      <h3>Join Team</h3>
      <form id="playerForm">
        <input
          type="text"
          id="ethAmount"
          placeholder="Amount to Wager (Ether). Min 0.0001"
        />
        <button type="button" onClick={() => register("Team1")}>
          Join Team1
        </button>
        <button type="button" onClick={() => register("Team2")}>
          Join Team2
        </button>
      </form>
      <div>
        <p>
          ETH Wagered:{" "}
          {ethWagered !== null
            ? `${ethers.formatEther(ethWagered)} ETH`
            : "Not set"}
        </p>
        <p>Team Requested: {teamRequested || "Not selected"}</p>
      </div>
      <hr />
      {ethWagered && ethers.parseEther("0.0001") <= ethWagered && (
        <>
          <h3>Enter Gamelord NodeId and Minecraft ID</h3>
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
          </form>
          <button type="button" onClick={() => joinTeamRequest(teamRequested)}>
            Join
          </button>
          <hr />
        </>
      )}
      <div>Chain ID: {chainId}</div>
      <div>Address: {userAccount}</div>
    </>
  );
};

export default Login;
