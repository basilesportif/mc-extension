package org.kinode;

import org.bukkit.plugin.java.JavaPlugin;
import org.bukkit.Bukkit;
import org.bukkit.Location;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.block.Action;
import org.bukkit.event.player.PlayerMoveEvent;
import org.bukkit.event.player.PlayerInteractEvent;
import org.bukkit.event.player.PlayerJoinEvent;
import org.bukkit.event.player.PlayerInteractEvent;

import org.kinode.MCKinodeWS;
import org.kinode.WorldInfo;
import java.net.URI;
import java.net.URISyntaxException;
import java.util.UUID;

public final class MCKinodePlugin extends JavaPlugin implements Listener {

    private static final String kinodeUri = "ws://localhost:8080/mcdriver:mcdriver:basilesex.os";
    private static MCKinodePlugin instance;
    private Location prevLocation;
    private static final int ALLOWED_BOUNDS = 50000;
    private boolean positionDisplayToggle = false;
    private MCKinodeWS client;

    // Add a field to track the last cube the player was in
    private String prevCube = "";

    @Override
    public void onEnable() {
        Bukkit.getPluginManager().registerEvents(this, this);
        instance = this;
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
        // Get the player who joined
        Player player = event.getPlayer();

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
    public void onKeyPress(PlayerInteractEvent event) {
        if (event.getAction() == Action.LEFT_CLICK_AIR) {
            positionDisplayToggle = !positionDisplayToggle;
            event.getPlayer().sendMessage("Position display is now " + (positionDisplayToggle ? "ON" : "OFF"));
        }
    }

    @EventHandler
    public void onPlayerMove(PlayerMoveEvent event) {
        Location toLocation = event.getTo();
        Player player = event.getPlayer();
        int cubeSize = 50; // Edge length of each cube

        // Calculate the base coordinates of the cube the player is in
        int baseX = (toLocation.getBlockX() / cubeSize) * cubeSize;
        int baseY = (toLocation.getBlockY() / cubeSize) * cubeSize;
        int baseZ = (toLocation.getBlockZ() / cubeSize) * cubeSize;

        // Calculate the center of the cube
        int centerX = baseX + cubeSize / 2;
        int centerY = baseY + cubeSize / 2;
        int centerZ = baseZ + cubeSize / 2;

        // Create a unique identifier for the cube
        String currentCube = "Base: " + baseX + "," + baseY + "," + baseZ + " Center: " + centerX + "," + centerY + "," + centerZ;

        // Check if the player has moved to a new cube
        if (!currentCube.equals(prevCube)) {
            getLogger().info("Player has moved to a new cube: " + currentCube);
            player.sendMessage("You are now in a new cube: " + currentCube);
            prevCube = currentCube; // Update the previous cube tracker
        }
    }

    /**
     * @return The Minecraft Plugin Instance
     */
    public static MCKinodePlugin getInstance() {
        return instance;
    }
}
