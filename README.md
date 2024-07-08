# mc-extension: A Kinode <-> Minecraft Interface
Configuration:
Minecraft version: 1.20.6 (Java Edition) (make sure to change the Minecraft version within the Minecraft Launcher)
Paper version: 1.20.6 (Minecraft Server hosting service): https://papermc.io/downloads/paper
Java version: 21 (OpenJDK 21): https://openjdk.java.net/install/


## Setup:
1. Have the Minecraft Launcher set to the correct version
2. Set up Paper MC server (https://docs.papermc.io/paper/getting-started) (remember to have the startup script in the same folder as the Paper jar file.)
3. Add plugin to the server (https://docs.papermc.io/paper/adding-plugins)
4. Start Kinode node (kit f) (for local development)
5. Install(kit bs gamelord) and (kit bs mcdriver)
6. Navigate to http://localhost:8080/gamelord:gamelord:basilesex.os (for local development).
7. Configure world (and add allowed players)
8. Start Minecraft Server (just write: `sh {name of startup script}` in the folder with the Paper Jar with the appropriate plugin jar in the server's plugin folder).
9. Join the server from the Minecraft Client (http://localhost:8080 should be the default URL).


### Note for the plugin
To compile a new version of the plugin, just run `mvn clean install` in the `mc-plugin` folder.
To update the plugin, just replace the old jar file (in the MC server plugin folder) with the new one.

### Note for using the world configuration generator:
To run, follow the `README.md` in the `ui` folder.
To load a previous world configuration, upload JSON file in the appropriate field.

### Java plugin code for Minecraft
Found in `mc-plugin`

### Kinode WASM program to talk to the Minecraft plugin
Found in `mcdriver`

### UI World Configuration Generator
Found in `ui`

### Gamelord WASM program
Found in `gamelord`
