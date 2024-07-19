# Playtesting Setup
## Planning Phase Setup
### Server Setup - gamelord

Needs to be constantly online for the game to work.

```bash
cd gamelord
kit bs
```

Open UI at localhost:8080/gamelord:gamelord:basilesex.os. 
Under "Edit Lobby" define the game name and server address. (these are only displayed to players, and not used for any logic.)
Under "Configure Points" define the spawn points and goal post.
These should be displayed during planning phase.

### Players Setup - mcclient

There can be as many mcclients as you want, and they dont need to be constantly online.

```bash
cd mcclient
kit bs
```

Open UI at localhost:8080/mcclient:mcclient:basilesex.os.
To join a team specify the server/gamelord node, i.e. uncentered.os (or whoever is running the server), and enter your minecraft ID which can be found in your minecraft launcher (both inputs are necessary and important for the game to work properly.)

After joining the team, you can start making changes in the map. After pressing enter, the changes will propagate to all other players. You should also be able to see other players' changes in real time.

You can only chat with your team, and the other team will not be able to see your messages.

### Experimenting with Maps

Currently, the map we are using is found in the /ui/public folder. If you want to experiment with a different map, all players should manually replace that folder with a new one containing the new map/world object.

This will be done automatically in the future.

## Playing Phase: Server Setup - A Kinode <-> Minecraft Interface

todo - pick correct minecraft version in game

Configuration:
Minecraft version: 1.20.6 (Java Edition) (make sure to change the Minecraft version within the Minecraft Launcher)
Paper version: 1.20.6 (Minecraft Server hosting service): https://papermc.io/downloads/paper
Java version minimum: 21 (OpenJDK 21): https://openjdk.java.net/install/


### Setup:
1. Have the Minecraft Launcher set to the correct version
2. Set up Paper MC server (https://docs.papermc.io/paper/getting-started) (remember to have the startup script in the same folder as the Paper jar file.)
3. Add plugin to the server (https://docs.papermc.io/paper/adding-plugins)
4. Start Kinode node (kit f) (for local development)
5. Install(kit bs gamelord) and (kit bs mcdriver)
6. Navigate to http://localhost:8080/gamelord:gamelord:basilesex.os (for local development).
7. Configure world (and add allowed players)
8. Start Minecraft Server (just write: `sh {name of startup script}` in the folder with the Paper Jar with the appropriate plugin jar in the server's plugin folder).
9. Join the server from the Minecraft Client (http://localhost:8080 should be the default URL).


#### Note for the plugin
To compile a new version of the plugin, just run `mvn clean install` in the `mc-plugin` folder.
To update the plugin, just replace the old jar file (in the MC server plugin folder) with the new one.

#### Note for using the world configuration generator:
To run, follow the `README.md` in the `ui` folder.
To load a previous world configuration, upload JSON file in the appropriate field.

#### Java plugin code for Minecraft
Found in `mc-plugin`

#### Kinode WASM program to talk to the Minecraft plugin
Found in `mcdriver`

#### UI World Configuration Generator
Found in `ui`

#### Gamelord WASM program
Found in `gamelord`
