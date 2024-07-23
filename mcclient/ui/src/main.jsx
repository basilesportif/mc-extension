import React from "react";
import ReactDOM from "react-dom/client";
import InterfaceUI from "./InterfaceUI";
import "./index.css";
import { MetaMaskUIProvider } from "@metamask/sdk-react-ui";

ReactDOM.createRoot(document.getElementById("root")).render(
  <React.StrictMode>
    <MetaMaskUIProvider
      sdkOptions={{
        dappMetadata: {
          name: "McClient App",
          url: window.location.href,
        },
        infuraAPIKey: "ce8167f5e4864cfd9f70e36c088bab16",
        // Other options.
      }}
    >
      <InterfaceUI />
    </MetaMaskUIProvider>
  </React.StrictMode>
);
