package org.kinode;

import org.bukkit.World;
import org.bukkit.plugin.java.JavaPlugin;
import org.bukkit.Bukkit;
import org.bukkit.Location;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.player.PlayerMoveEvent;
import org.bukkit.event.player.PlayerInteractEvent;
import org.bukkit.event.player.PlayerJoinEvent;
import org.bukkit.event.player.PlayerInteractEvent;
import org.bukkit.event.block.Action;

import org.kinode.MCKinodeWS;
import org.kinode.WorldInfo;
import java.net.URI;
import java.net.URISyntaxException;
import java.util.UUID;

public final class MCKinodePlugin extends JavaPlugin implements Listener {

    private static final String kinodeUri = "ws://localhost:8080/mcdriver:mcdriver:astronaut.os";
    private static MCKinodePlugin instance;
    private Location prevLocation;
    private static final int ALLOWED_BOUNDS = 50000;
    private boolean positionDisplayToggle = false;
    private MCKinodeWS client;

    // Define a fixed reference point for the "world center"

    private static final Location WORLD_CENTER = new Location(null, 0, 0, 0);

    // Add a field to track the last cube the player was in
    private String prevCube = "";

    @Override
    public void onEnable() {
        Bukkit.getPluginManager().registerEvents(this, this);
        instance = this;
        // Set the server's spawn location
        //World world = Bukkit.getWorlds().get(0); // Get the first world
        //world.setSpawnLocation(50, 50, 50); // Set the spawn location
        getLogger().info("Server spawn location set to: 6, 111, 2");
        getLogger().info("STARTING KINODE <-> MC INTERFACE PLUGIN");
        try {
            client = new MCKinodeWS(new URI(kinodeUri));
            client.connect();
        } catch (URISyntaxException e) {
            e.printStackTrace();
        }

        /* diagnostic prints */
        // WorldInfo.printWorldInfo();
        // WorldInfo.printMapInfo();

        if (client.isConnected()) {
            getLogger().info("Connected to Kinode WS process");
        } else {
            getLogger().info("Failed to connect to Kinode WS process");
        }
    }

    @Override
    public void onDisable() {
        // Plugin shutdown logic
        // here
    }

    @EventHandler
    public void onPlayerJoin(PlayerJoinEvent event) {
        Player player = event.getPlayer();
        Location playerLocation = player.getLocation();

        // Calculate the player's position relative to the world center
        int playerX = playerLocation.getBlockX();
        int playerY = playerLocation.getBlockY();
        int playerZ = playerLocation.getBlockZ();

        // Set the initial cube based on the player's position
        setInitialCube(player, playerX, playerY, playerZ);

        // Get the player's UUID
        UUID playerUUID = player.getUniqueId();

        // Print the UUID to the console
        getLogger().info("Player UUID: " + playerUUID.toString());
        getLogger().info("Fetching player id from Mojang API...");
        String playerName = MojangAPI.getPlayerId(playerUUID);
        getLogger().info("Player name: " + playerName);

        // Send WebSocket message
        if (client != null && client.isConnected()) {
            client.sendPlayerJoinMessage(playerName, event);
        }
    }


    @EventHandler
    public void onPlayerMove(PlayerMoveEvent event) {
        Location toLocation = event.getTo();
        Player player = event.getPlayer();

        // Calculate the player's position relative to the world center
        int playerX = toLocation.getBlockX();
        int playerY = toLocation.getBlockY();
        int playerZ = toLocation.getBlockZ();

        // Check if the player has moved to a new cube
        checkAndUpdateCube(player, playerX, playerY, playerZ);
    }

    private void setInitialCube(Player player, int playerX, int playerY, int playerZ) {
        int cubeSize = 16;

        // Adjust player's coordinates relative to the world center
        int adjustedX = playerX - WORLD_CENTER.getBlockX();
        int adjustedY = playerY - WORLD_CENTER.getBlockY();
        int adjustedZ = playerZ - WORLD_CENTER.getBlockZ();

        // Calculate the base coordinates of the cube the player is in
        int baseX = (int) Math.floor((double) adjustedX / cubeSize) * cubeSize;
        int baseY = (int) Math.floor((double) adjustedY / cubeSize) * cubeSize;
        int baseZ = (int) Math.floor((double) adjustedZ / cubeSize) * cubeSize;

        // Calculate the center of the cube
        int centerX = baseX + (cubeSize / 2) + WORLD_CENTER.getBlockX();
        int centerY = baseY + (cubeSize / 2) + WORLD_CENTER.getBlockY();
        int centerZ = baseZ + (cubeSize / 2) + WORLD_CENTER.getBlockZ();

        // Set the initial cube
        String initialCube = "Center: " + centerX + "," + centerY + "," + centerZ;
        prevCube = initialCube;

        getLogger().info("Initial cube set to: " + initialCube);
        player.sendMessage("Welcome! You're starting at the cube: " + initialCube);
    }

    private void checkAndUpdateCube(Player player, int playerX, int playerY, int playerZ) {
        int cubeSize = 16;

        // Adjust player's coordinates relative to the world center
        int adjustedX = playerX - WORLD_CENTER.getBlockX();
        int adjustedY = playerY - WORLD_CENTER.getBlockY();
        int adjustedZ = playerZ - WORLD_CENTER.getBlockZ();

        // Calculate the base coordinates of the cube the player is in
        int baseX = (int) Math.floor((double) adjustedX / cubeSize) * cubeSize;
        int baseY = (int) Math.floor((double) adjustedY / cubeSize) * cubeSize;
        int baseZ = (int) Math.floor((double) adjustedZ / cubeSize) * cubeSize;

        // Calculate the center of the cube
        int centerX = baseX + (cubeSize / 2) + WORLD_CENTER.getBlockX();
        int centerY = baseY + (cubeSize / 2) + WORLD_CENTER.getBlockY();
        int centerZ = baseZ + (cubeSize / 2) + WORLD_CENTER.getBlockZ();

        // Create a unique identifier for the cube
        String currentCube = "Center: " + centerX + "," + centerY + "," + centerZ;

        // Check if the player has moved to a new cube
        if (!currentCube.equals(prevCube)) {
            getLogger().info("Player has moved to a new cube: " + currentCube);
            player.sendMessage("You are now in a new cube: " + currentCube);
            prevCube = currentCube; // Update the previous cube tracker            // Send ValidateMove message
            if (client != null && client.isConnected()) {
                client.sendValidateMoveMessage(player.getName(), centerX, centerY, centerZ);
            }
        }
    }

    /**
     * @return The Minecraft Plugin Instance
     */
    public static MCKinodePlugin getInstance() {
        return instance;
    }
}