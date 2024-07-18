import React from "react";
import { Message } from "@chatscope/chat-ui-kit-react";

const Msg = ({ message }) => {
  return (
    <div
      style={{ display: "flex", flexDirection: "row", alignItems: "center" }}
    >
      <div style={{ marginRight: "10px", fontSize: "0.8em" }}>
        {new Date(message.time * 1000).toLocaleTimeString()}
      </div>
      <div style={{ marginRight: "10px", fontSize: "0.8em" }}>
        {message.from.kinode_id}:
      </div>
      <Message model={{ message: message.msg }}></Message>
    </div>
  );
};

export default Msg;
