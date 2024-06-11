# MC Kinode Interface
_Minecraft Java Plugin Template using Maven and PaperMC API_


## Features

- **ItemsUtils**: A class that contains methods to create ItemStacks easily

## How to use

1. Clone this repository
2. Open the project in your IDE
3. Change the package name to your own
4. Change the plugin name in the plugin.yml
5. Change the plugin details in the pom.xml
6. Start coding!

## How to build

1. Run `mvn clean install` or use the maven build tool in your IDE
2. The jar file will be in the target folder

## Use ItemsUtils

Create an ItemStack with a name and a lore
```java
ItemStack item = Items.createItem(Material.DIAMOND, 1, "My Item", Arrays.asList("My Lore"));
```

Create an Skull item with a name and a lore
```java
ItemStack item = Items.createSkull(Material.DIAMOND, 1, "My Item", Arrays.asList("My Lore"), "Notch");
```

## Commands

Register your command in the PluginMgr class inside the registerCommands() method
```java
public class PluginMgr extends Manager {
    
    ..
    private void registerCommands() {
        PluginInstance.getInstance().getCommand("mycommand").setExecutor(new MyCommand());
    }
    ..
}
```

## Dependencies

- **PaperMC API**: [https://papermc.io/](https://papermc.io/)
- **Maven**: [https://maven.apache.org/](https://maven.apache.org/)
- **Java 8**: [https://www.java.com/en/download/](https://www.java.com/en/download/)

