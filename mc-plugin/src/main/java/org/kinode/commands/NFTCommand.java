package org.kinode.commands;

import org.bukkit.Bukkit;
import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;

public class NFTCommand implements CommandExecutor {
    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {

        Player player = (Player) sender;

        if (!player.hasPermission("mcplugin.nft")) {
            player.sendMessage("You do not have permission to use this command.");
            return true;
        }

        if (args.length == 0) {
            player.sendMessage("Usage: /nft <subcommand>");
            return true;
        }

        String subCommand = args[0];

        // Example: /nft list
        if (subCommand.equalsIgnoreCase("list")) {
            // Execute the imageframe list command on behalf of the player
            Bukkit.dispatchCommand(Bukkit.getConsoleSender(), "imageframe list");
            player.sendMessage("Executed /imageframe list");
        }

        // Example: /nft get <name>
        if (subCommand.equalsIgnoreCase("get") && args.length == 2) {
            String name = args[1];
            // Execute the imageframe get command on behalf of the player
            Bukkit.dispatchCommand(Bukkit.getConsoleSender(), "imageframe get " + name);
            player.sendMessage("Executed /imageframe get " + name);
        }

        return true;
    }
}