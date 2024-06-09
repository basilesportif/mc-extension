package org.kinode.misc;

import org.bukkit.Material;
import org.bukkit.inventory.ItemStack;
import org.bukkit.inventory.meta.ItemMeta;
import org.bukkit.inventory.meta.SkullMeta;

import java.util.List;

public class Items {

    /**
     * @param material The material of the item
     * @param amount   The amount of the item
     * @param name     The name of the item
     * @param lore     The lore of the item
     * @return The item
     *         Create an ItemStack with metadata
     */
    public static ItemStack create(Material material, int amount, String name, List<String> lore) {
        ItemStack item = new ItemStack(material, amount);
        ItemMeta meta = item.getItemMeta();
        meta.setDisplayName(name);
        if (lore != null)
            meta.setLore(lore);
        item.setItemMeta(meta);
        return item;
    }

    /**
     * @param material The material of the item
     * @param amount   The amount of the item
     * @param name     The name of the item
     * @param lore     The lore of the item
     * @param owner    The owner of the skull
     * @return The item
     *         Create an (Skull item) ItemStack with metadata
     */
    public static ItemStack createSkull(Material material, int amount, String name, List<String> lore, String owner) {
        ItemStack item = new ItemStack(material, amount);
        SkullMeta meta = (SkullMeta) item.getItemMeta();

        meta.setDisplayName(name);
        if (lore != null)
            meta.setLore(lore);
        item.setItemMeta(meta);
        meta.setOwner(owner);
        return item;
    }
}
