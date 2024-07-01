package org.kinode;

import org.json.JSONObject;
import org.json.JSONArray;
import org.json.JSONException;
import org.bukkit.Bukkit;
import org.bukkit.event.player.PlayerJoinEvent;
import org.bukkit.Location; // Import the Location class

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
        //send("Hello, it is me. Mario :)");
        //send("\"SanityCheck\"");
        System.out.println("new connection opened");
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
                + "\"player\": {"
                + "\"kinode_id\": \"fake2.dev\","
                + "\"minecraft_player_name\": \"" + playerName + "\""
                + "}"
                + "}"
                + "}"
                + "}";
        send(message);

        // Handle the response
        onMessageResponse = (response) -> {
            JSONObject jsonResponse = new JSONObject(response);
            JSONArray addPlayerArray = jsonResponse.getJSONArray("AddPlayer");
            if (addPlayerArray.length() > 0) {
                boolean success = addPlayerArray.getBoolean(0);
                String messageResponse = addPlayerArray.getString(1);
                JSONObject cubeData = addPlayerArray.getJSONObject(2);

                JSONArray center = cubeData.getJSONArray("center");
                int x = center.getInt(0);
                int y = center.getInt(1);
                int z = center.getInt(2);

                if (success) {
                    MCKinodePlugin.getInstance().getLogger().info("Player join allowed: " + messageResponse);
                    Bukkit.getScheduler().runTask(MCKinodePlugin.getInstance(), () -> {
                        Location spawnLocation = new Location(event.getPlayer().getWorld(), x, y, z);
                        event.getPlayer().teleport(spawnLocation);
                    });
                } else {
                    MCKinodePlugin.getInstance().getLogger().info("Player join denied: " + messageResponse);
                    Bukkit.getScheduler().runTask(MCKinodePlugin.getInstance(), () -> {
                        event.getPlayer().kickPlayer("You are not allowed to join the server.");
                    });
                }
            }
        };
    }

    // Define the WebSocketResponseHandler interface here
    public interface WebSocketResponseHandler {
        void handleResponse(String response);
    }
}
