package org.kinode;

import org.bukkit.Bukkit;
import org.bukkit.Location;
import org.bukkit.NamespacedKey;
import org.bukkit.Server;
import org.bukkit.World;
import org.bukkit.WorldBorder;

// for map
import org.bukkit.inventory.ItemStack;
import org.bukkit.Material;
import org.bukkit.inventory.meta.MapMeta;
import org.bukkit.map.MapView;
import org.bukkit.persistence.PersistentDataContainer;
import org.bukkit.persistence.PersistentDataType;

public class WorldInfo {
  public static void printWorldInfo() {
    Server server = Bukkit.getServer();
    World world = server.getWorld("world");
    if (world != null) {
      WorldBorder border = world.getWorldBorder();
      Location center = border.getCenter();
      double centerX = center.getX();
      double centerY = center.getY();
      double centerZ = center.getZ();
      double size = border.getSize();
      System.out.println("Map Center Coordinates: X=" + centerX + ", Y=" + centerY + ", Z=" + centerZ);
      System.out.println("World Border Size: " + size);
    }
  }

  public static void printMapInfo() {
    ItemStack mapItem = new ItemStack(Material.MAP);
    if (mapItem == null || !(mapItem.getItemMeta() instanceof MapMeta)) {
      throw new IllegalArgumentException("ItemStack is not a map");
    }

    MapMeta mapMeta = (MapMeta) mapItem.getItemMeta();
    MapView mapView = mapMeta.getMapView();

    if (mapView == null) {
      throw new IllegalStateException("MapView is null");
    }

    // The zoom level is stored in the "scale" tag
    PersistentDataContainer dataContainer = mapMeta.getPersistentDataContainer();
    Integer zoomLevel = dataContainer.get(new NamespacedKey("minecraft", "map/scale"), PersistentDataType.INTEGER);

    if (zoomLevel == null) {
      throw new IllegalStateException("Zoom level not found in map NBT data");
    }

    MCKinodePlugin.getInstance().getLogger().info("Map Zoom Level: " + zoomLevel);

  }
}