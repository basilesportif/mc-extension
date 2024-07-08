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
                + "\"PlayerJoinRequest\": {"
                + "\"minecraft_player_name\": \"" + playerName + "\""
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

            if (jsonResponse.has("AddPlayer")) {
                responseArray = jsonResponse.getJSONArray("AddPlayer");
                success = responseArray.getBoolean(0);
                messageResponse = responseArray.getString(1);
            } else if (jsonResponse.has("AddPlayerFailed")) {
                responseArray = jsonResponse.getJSONArray("AddPlayerFailed");
                success = responseArray.getBoolean(0);
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
                + "\"ValidateMove\": {"
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

            if (jsonResponse.has("ValidateMove")) {
                JSONArray responseArray = jsonResponse.getJSONArray("ValidateMove");
                success = responseArray.getBoolean(0);
                messageResponse = responseArray.getString(1);

                if (success) {
                    MCKinodePlugin.getInstance().getLogger().info("Move allowed: " + messageResponse);
                    Player player = Bukkit.getPlayer(playerName);
                    player.sendMessage("Move allowed: " + messageResponse);
                } else {
                    MCKinodePlugin.getInstance().getLogger().info("Move invalid: " + messageResponse);
                    Bukkit.getScheduler().runTask(MCKinodePlugin.getInstance(), () -> {
                        Player player = Bukkit.getPlayer(playerName);
                        if (player != null) {
                            player.sendMessage("Move not allowed: " + messageResponse);
                            
                            // Get the player's current location
                            Location currentLocation = player.getLocation();
                            
                            // Calculate a new position 15 blocks behind the player
                            Vector direction = currentLocation.getDirection().normalize().multiply(-10);  // Reverse direction and move 15 blocks back
                            Location newLocation = currentLocation.add(direction);
                            
                            // Ensure the new location is safe (not inside a block)
                            while (newLocation.getBlock().getType().isSolid() && newLocation.getY() < 256) {
                                newLocation.add(0, 1, 0);  // Move up until we find a non-solid block
                            }
                            
                            // Teleport the player to the new location
                            player.teleport(newLocation);
                            player.sendMessage("You've been moved back to a safe location.");
                        }
                    });
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
