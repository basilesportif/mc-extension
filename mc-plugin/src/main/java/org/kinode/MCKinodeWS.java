package org.kinode;

import org.kinode.MCKinodePlugin;

import java.net.URI;
import java.net.URISyntaxException;
import java.nio.ByteBuffer;

import org.java_websocket.client.WebSocketClient;
import org.java_websocket.drafts.Draft;
import org.java_websocket.drafts.Draft_6455;
import org.java_websocket.handshake.ServerHandshake;

public class MCKinodeWS extends WebSocketClient {

  private boolean isConnected = false;

  public MCKinodeWS(URI serverUri, Draft draft) {
    super(serverUri, draft);
  }

  public MCKinodeWS(URI serverURI) {
    super(serverURI, new Draft_6455());
  }

  public boolean isConnected() {
    return isConnected;
  }

  @Override
  public void onOpen(ServerHandshake handshakedata) {
    isConnected = true;
    send("Hello, it is me. Mario :)");
    send("\"SanityCheck\"");
    System.out.println("new connection opened");

  }

  @Override
  public void onClose(int code, String reason, boolean remote) {
    MCKinodePlugin.getInstance().getLogger()
        .info("MC-Kinode WS closed with exit code " + code + " additional info: " + reason);
  }

  @Override
  public void onMessage(String message) {
    // TODO: use singleton pattern to send messages to the PluginInstance
    System.out.println("received message: " + message);
  }

  @Override
  public void onMessage(ByteBuffer message) {
    System.out.println("received ByteBuffer");
  }

  @Override
  public void onError(Exception ex) {
    System.err.println("an error occurred:" + ex);
  }
  /*
   * public static void main(String[] args) throws URISyntaxException {
   * WebSocketClient client = new EmptyClient(new URI("ws://localhost:8887"));
   * client.connect();
   * }
   */
}