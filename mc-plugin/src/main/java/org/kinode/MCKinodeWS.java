package org.kinode;

import org.json.JSONObject;
import org.json.JSONArray;
import org.json.JSONException;
import org.bukkit.Bukkit;
import org.bukkit.event.player.PlayerJoinEvent;
import org.bukkit.Location; // Import the Location class
import org.bukkit.World;
import org.bukkit.entity.Player;
import org.bukkit.util.Vector;

import java.net.URI;
import java.nio.ByteBuffer;

import org.java_websocket.client.WebSocketClient;
import org.java_websocket.drafts.Draft;
import org.java_websocket.drafts.Draft_6455;
import org.java_websocket.handshake.ServerHandshake;

public class MCKinodeWS extends WebSocketClient {

    private boolean isConnected = false;
    public WebSocketResponseHandler onMessageResponse;

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
        System.out.println("new connection opened");
        send("{\"type\":\"Connection established\"}");
    }

    @Override
    public void onClose(int code, String reason, boolean remote) {
        MCKinodePlugin.getInstance().getLogger()
                .info("MC-Kinode WS closed with exit code " + code + " additional info: " + reason);
    }

    @Override
    public void onMessage(String message) {
        System.out.println("received message: " + message);
        if (onMessageResponse != null) {
            onMessageResponse.handleResponse(message);
        }
    }

    @Override
    public void onMessage(ByteBuffer message) {
        System.out.println("received ByteBuffer");
    }

    @Override
    public void onError(Exception ex) {
        System.err.println("an error occurred:" + ex);
    }

    public void sendPlayerJoinMessage(String playerName, PlayerJoinEvent event) {
        String message = "{"
                + "\"type\": \"WebSocketPush\","
                + "\"channel_id\": 1,"
                + "\"message_type\": \"Text\","
                + "\"body\": {"
                + "\"PlayerSpawnRequest\": {"
                + "\"minecraft_id\": \"" + playerName + "\""
                + "}"
                + "}"
                + "}";
        send(message);

        // Handle the response
        onMessageResponse = (response) -> {
            JSONObject jsonResponse = new JSONObject(response);
            JSONArray responseArray;
            boolean success;
            String messageResponse;

            if (jsonResponse.has("PlayerSpawnRequestAuthorized")) {
                responseArray = jsonResponse.getJSONArray("PlayerSpawnRequestAuthorized");
                success = responseArray.getBoolean(0);
                messageResponse = responseArray.getString(1);
                //remember that the third element should be the cube that the player spawns in
            } else if (jsonResponse.has("PlayerSpawnRequestDenied")) {
                responseArray = jsonResponse.getJSONArray("PlayerSpawnRequestDenied");
                success = responseArray.getBoolean(0);
                // remember that the th
                messageResponse = responseArray.getString(1);
            } else {
                System.err.println("Unexpected response format: " + response);
                return;
            }

            if (success) {
                JSONObject cubeData = responseArray.getJSONObject(2);
                JSONArray center = cubeData.getJSONArray("center");
                int x = center.getInt(0);
                int y = center.getInt(1);
                int z = center.getInt(2);
                MCKinodePlugin.getInstance().getLogger().info("Player join allowed: " + messageResponse);
            } else {
                MCKinodePlugin.getInstance().getLogger().info("Player join denied: " + messageResponse);
                Bukkit.getScheduler().runTask(MCKinodePlugin.getInstance(), () -> {
                    event.getPlayer().kickPlayer("You are not allowed to join the server.");
                });
            }
        };
    }

    public void sendValidateMoveMessage(String playerName, int x, int y, int z) {
        String message = "{"
                + "\"type\": \"WebSocketPush\","
                + "\"channel_id\": 1,"
                + "\"message_type\": \"Text\","
                + "\"body\": {"
                + "\"CubeTransitionRequest\": {"
                + "\"minecraft_id\": \"" + playerName + "\","
                + "\"cube\": {"
                + "\"center\": [" + x + ", " + y + ", " + z + "],"
                + "\"side_length\": 50"
                + "}"
                + "}"
                + "}"
                + "}";
        send(message);
        System.out.println("Sent ValidateMove message: " + message);

        onMessageResponse = (response) -> {
            JSONObject jsonResponse = new JSONObject(response);
            boolean success;
            String messageResponse;
            if (jsonResponse.has("TransitionTriggeredResponse")) {
                JSONObject transitionTriggeredResponse = jsonResponse.getJSONObject("TransitionTriggeredResponse");
                JSONArray effectsArray = transitionTriggeredResponse.getJSONArray("effects");
                MCKinodePlugin.getInstance().getLogger().info("TransitionTriggeredResponse effects: " + effectsArray.toString());
                Player player = Bukkit.getPlayer(playerName);
                if (player != null) {
                    player.sendMessage("Entering new territory with effects: " + effectsArray.toString());
                }
            } else if (jsonResponse.has("TransitionSilentResponse")) {
                MCKinodePlugin.getInstance().getLogger().info("Entering non-enemy territory");
                Player player = Bukkit.getPlayer(playerName);
                if (player != null) {
                    player.sendMessage("You are entering non-enemy territory");
                }
            } else {
                System.err.println("Unexpected response format: " + response);
            }
        };
        
    }

    // Define the WebSocketResponseHandler interface here
    public interface WebSocketResponseHandler {
        void handleResponse(String response);
    }
}
