import React, { useEffect, useState } from "react";
import { ethers, parseEther } from "ethers";
import Counter from "./abi/Counter.json";

let signer = null;
let provider;
const CONTRACT_ADDRESS = "0x0B306BF915C4d645ff596e518fAf3F9669b97016";
const Login = ({ ourInTeam, setOurInTeam }) => {
  const [userAccount, setUserAccount] = useState("");
  const [chainId, setChainId] = useState();
  const [contract, setContract] = useState(null);
  const [number, setNumber] = useState();

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
        const counter_contract = new ethers.Contract(
          CONTRACT_ADDRESS,
          Counter.abi,
          signer
        );
        setContract(counter_contract);
      }
    };
    loadEthers();
  }, []);

  const increment = async () => {
    try {
      let tx = await contract.increment();
      let receipt = await tx.wait();
      console.log("TX", receipt);
    } catch (error) {
      console.error(error);
    }
  };
  const getNumber = async () => {
    try {
      let number = await contract.number();
      console.log("NUMBER", number);
      setNumber(number.toString());
    } catch (error) {
      console.error(error);
    }
  };
  const sendEth = async () => {
    try {
      let tx = await signer.sendTransaction({
        to: "0xc8637aadB7619fcaF1aD108682cF593cD124D499",
        value: parseEther("0.1"),
      });
      let receipt = await tx.wait();

      console.log(receipt);
    } catch (error) {
      console.error(error);
    }
  };

  return (
    <>
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
      <hr />
      <div>Chain ID: {chainId}</div>
      <div>Address: {userAccount}</div>
      <button className="incrementButton" onClick={increment}>
        Increment
      </button>
      <button className="getNumberButton" onClick={getNumber}>
        Get Number
      </button>
      <div>Number: {number}</div>
      <button className="sendEthButton" onClick={sendEth}>
        Send ETH
      </button>
    </>
  );
};

export default Login;
