import React, { useEffect, useState, useSyncExternalStore } from "react";
import { MetaMaskButton } from "@metamask/sdk-react-ui";

export const formatBalance = (rawBalance) => {
    const balance = (parseInt(rawBalance) / 1000000000000000000).toFixed(2)
    return balance
  }
  
  export const formatChainAsNum = (chainIdHex) => {
    const chainIdNum = parseInt(chainIdHex)
    return chainIdNum
  }
  
  export const formatAddress = (addr) => {
    const upperAfterLastTwo = addr.slice(0, 2) + addr.slice(2)
    return `${upperAfterLastTwo.substring(0, 5)}...${upperAfterLastTwo.substring(39)}`
  }


let providers = [];
const subscribe = (callback) => {
  function onAnnouncement(event) {
    if (providers.map((p) => p.info.uuid).includes(event.detail.info.uuid)) {
      return;
    }
    providers = [...providers, event.detail];
    callback();
  }

  // Listen for custom event
  window.addEventListener("eip6963:announceProvider", onAnnouncement);

  // Dispatch the event, which triggers the event listener in the MetaMask wallet.
  window.dispatchEvent(new Event("eip6963:requestProvider"));

  // Return a function to remove the event listener
  return () =>
    window.removeEventListener("eip6963:announceProvider", onAnnouncement);
};
const getSnapshot = () => providers;
const getServerSnapshot = () => [];
const useSyncProviders = () => {
  const [cachedProviders, setCachedProviders] = useState(getSnapshot);

  useEffect(() => {
    const unsubscribe = subscribe(() => {
      setCachedProviders(getSnapshot());
    });
    return unsubscribe;
  }, []);

  return cachedProviders;
};

const Login = ({ ourInTeam, setOurInTeam }) => {
  const [selectedWallet, setSelectedWallet] = useState();
  const [userAccount, setUserAccount] = useState("");
  const [chainId, setChainId] = useState();
  const providers = useSyncProviders();

  const handleConnect = async (providerWithInfo) => {
    try {
      const accounts = await providerWithInfo.provider.request({
        method: "eth_requestAccounts",
      });

      setSelectedWallet(providerWithInfo);
      setUserAccount(accounts?.[0]);
    } catch (error) {
      console.error(error);
    }
  };

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
    network(selectedWallet);
  }, [selectedWallet]);

  const network = async (providerWithInfo) => {
    try {
      const chainId = await providerWithInfo.provider.request({
        method: "eth_chainId",
      });
      console.log("Connected to chain:", chainId);
      setChainId(chainId);
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
      <h3>Connect with MetaMask</h3>
      {/* <ConnectWallet provider={provider}/> */}
      <h2>Wallets Detected:</h2>
      <div>
        {providers.length > 0 ? (
          providers?.map((provider) => (
            <button
              key={provider.info.uuid}
              onClick={() => handleConnect(provider)}
            >
              <img src={provider.info.icon} alt={provider.info.name} />
              <div>{provider.info.name}</div>
            </button>
          ))
        ) : (
          <div>No Announced Wallet Providers</div>
        )}
      </div>
      <hr />
      <h2>{userAccount ? "" : "No "}Wallet Selected</h2>
      {userAccount && (
        <div>
          <div>
            <img
              src={selectedWallet.info.icon}
              alt={selectedWallet.info.name}
            />
            <div>{selectedWallet.info.name}</div>
            <div>({formatAddress(userAccount)})</div>
          </div>
        </div>
      )}
      <hr />
      <h2>Network Information</h2>
      <div>Chain ID: {chainId}</div>
    </>
  );
};

export default Login;
