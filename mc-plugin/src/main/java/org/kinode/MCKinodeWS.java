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
import org.bukkit.entity.EntityType;

import java.net.URI;
import java.nio.ByteBuffer;

import org.java_websocket.client.WebSocketClient;
import org.java_websocket.drafts.Draft;
import org.java_websocket.drafts.Draft_6455;
import org.java_websocket.handshake.ServerHandshake;

import org.bukkit.potion.PotionEffect;
import org.bukkit.potion.PotionEffectType;
import org.bukkit.Particle;

import org.bukkit.block.Block;
import org.bukkit.block.BlockFace;

import java.util.HashMap;
import java.util.Map;

public class MCKinodeWS extends WebSocketClient {

    private boolean isConnected = false;
    // Remove the onMessageResponse field
    // public WebSocketResponseHandler onMessageResponse;

    // Add a map to store player-specific response handlers
    private Map<String, WebSocketResponseHandler> playerResponseHandlers = new HashMap<>();

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
        // Assuming the response is related to the last sent request
        for (Map.Entry<String, WebSocketResponseHandler> entry : playerResponseHandlers.entrySet()) {
            entry.getValue().handleResponse(message);
            break; // Handle only the first matching entry
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

        // Create a player-specific response handler
        WebSocketResponseHandler responseHandler = (response) -> {
            JSONObject jsonResponse = new JSONObject(response);
            JSONArray responseArray;
            boolean success;
            String messageResponse;
            JSONObject centerObject = null; // Initialize centerObject with null

            if (jsonResponse.has("PlayerSpawnRequestAuthorized")) {
                responseArray = jsonResponse.getJSONArray("PlayerSpawnRequestAuthorized");
                success = responseArray.getBoolean(0);
                messageResponse = responseArray.getString(1);
                centerObject = responseArray.getJSONObject(2); // Get the JSONObject at index 2

                if (success) {
                    JSONArray centerArray = centerObject.getJSONArray("center");
                    int x = centerArray.getInt(0);
                    int y = centerArray.getInt(1);
                    int z = centerArray.getInt(2);
                    MCKinodePlugin.getInstance().getLogger().info("Player join allowed: " + messageResponse);
                    MCKinodePlugin.getInstance().getLogger().info("Spawn point: (" + x + ", " + y + ", " + z + ")");

                    Bukkit.getScheduler().runTask(MCKinodePlugin.getInstance(), () -> {
                        event.getPlayer().teleport(new Location(event.getPlayer().getWorld(), x, y, z));
                    });
                } else {
                    MCKinodePlugin.getInstance().getLogger().info("Player join denied: " + messageResponse);
                    Bukkit.getScheduler().runTask(MCKinodePlugin.getInstance(), () -> {
                        event.getPlayer().kickPlayer("You are not allowed to join the server.");
                    });
                }
            } else if (jsonResponse.has("PlayerSpawnRequestDenied")) {
                responseArray = jsonResponse.getJSONArray("PlayerSpawnRequestDenied");
                success = responseArray.getBoolean(0);
                messageResponse = responseArray.getString(1);
            } else {
                System.err.println("Unexpected response format: " + response);
                return;
            }

            // Check if centerObject is not null before using it
            if (success && centerObject != null) {
                JSONArray centerArray = centerObject.getJSONArray("center");
                int x = centerArray.getInt(0);
                int y = centerArray.getInt(1);
                int z = centerArray.getInt(2);
                MCKinodePlugin.getInstance().getLogger().info("Player join allowed: " + messageResponse);
                MCKinodePlugin.getInstance().getLogger().info("Spawn point: (" + x + ", " + y + ", " + z + ")");

                Bukkit.getScheduler().runTask(MCKinodePlugin.getInstance(), () -> {
                    event.getPlayer().teleport(new Location(event.getPlayer().getWorld(), x, y, z));
                });
            } else {
                MCKinodePlugin.getInstance().getLogger().info("Player join denied: " + messageResponse);
                Bukkit.getScheduler().runTask(MCKinodePlugin.getInstance(), () -> {
                    event.getPlayer().kickPlayer("You are not allowed to join the server.");
                });
            }
        };

        // Store the response handler in the map
        playerResponseHandlers.put(playerName, responseHandler);
    }
    //change this to SendCubeTransitionRequest
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
                + "\"side_length\": 16"
                + "}"
                + "}"
                + "}"
                + "}";
        send(message);
        System.out.println("Sent ValidateMove message: " + message);

        // Create a player-specific response handler
        WebSocketResponseHandler responseHandler = (response) -> {
            try {
                JSONObject jsonResponse = new JSONObject(response);
                if (jsonResponse.has("TransitionTriggeredResponse")) {
                    JSONArray transitionTriggeredResponse = jsonResponse.getJSONArray("TransitionTriggeredResponse");
                    String minecraftId = transitionTriggeredResponse.getString(0);
                    // The second element is now a JSONObject instead of a JSONArray
                    JSONObject effectsObject = transitionTriggeredResponse.getJSONObject(1);
                    JSONArray effectsArray = effectsObject.getJSONArray("effects");
                    MCKinodePlugin.getInstance().getLogger().info("TransitionTriggeredResponse effects: " + effectsArray.toString());
                    Bukkit.getScheduler().runTask(MCKinodePlugin.getInstance(), () -> {
                        Player player = Bukkit.getPlayer(minecraftId);
                        if (player != null) {
                            player.sendMessage("Entering new territory with effects: " + effectsArray.toString());

                            for (int i = 0; i < effectsArray.length(); i++) {
                                String effectName = effectsArray.getString(i);
                                PotionEffectType effectType = PotionEffectType.getByName(effectName);
                                if (effectType != null) {
                                    PotionEffect effect = new PotionEffect(effectType, 200, 1);
                                    player.addPotionEffect(effect);

                                    player.getWorld().spawnParticle(Particle.SPELL_WITCH, player.getLocation().add(0, 1, 0), 50, 0.5, 0.5, 0.5, 0.1);
                                } else {
                                    MCKinodePlugin.getInstance().getLogger().warning("Unknown effect: " + effectName);
                                }
                            }
                        }
                    });
                } else if (jsonResponse.has("TransitionSilentResponse")) {
                    JSONArray transitionSilentResponse = jsonResponse.getJSONArray("TransitionSilentResponse");
                    String minecraftId = transitionSilentResponse.getString(0);
                    String message_response = transitionSilentResponse.getString(1);
                    MCKinodePlugin.getInstance().getLogger().info("TransitionSilentResponse: " + message_response);
                    Bukkit.getScheduler().runTask(MCKinodePlugin.getInstance(), () -> {
                        Player player = Bukkit.getPlayer(minecraftId);
                        if (player != null) {
                            player.sendMessage(message_response);
                        }
                    });
                } else if (jsonResponse.has("GameOver")) {
                    String winningTeam = jsonResponse.getString("GameOver");
                    MCKinodePlugin.getInstance().getLogger().info("Game Over! Winning team: " + winningTeam);
                    Bukkit.getScheduler().runTask(MCKinodePlugin.getInstance(), () -> {
                        for (Player player : Bukkit.getOnlinePlayers()) {
                            player.sendTitle("§c§lGame Over!", "§eWinning Team: §f" + winningTeam, 10, 70, 20);
                        }
                    });
                } else if (jsonResponse.has("TransitionDeniedResponse")) {
                    JSONArray transitionDeniedResponse = jsonResponse.getJSONArray("TransitionDeniedResponse");
                    String minecraftId = transitionDeniedResponse.getString(0);
                    MCKinodePlugin.getInstance().getLogger().info("TransitionDeniedResponse for player: " + minecraftId);
                    Bukkit.getScheduler().runTask(MCKinodePlugin.getInstance(), () -> {
                        Player player = Bukkit.getPlayer(minecraftId);
                        if (player != null) {
                            MCKinodePlugin.getInstance().teleportToPreviousCube(player);
                        }
                    });
                } else {
                    System.err.println("Unexpected JSON response format: " + response);
                }
            } catch (JSONException e) {
                System.err.println("Failed to parse JSON response: " + response);
            }
        };

        // Store the response handler in the map
        playerResponseHandlers.put(playerName, responseHandler);
    }

    // Define the WebSocketResponseHandler interface here
    public interface WebSocketResponseHandler {
        void handleResponse(String response);
    }
}