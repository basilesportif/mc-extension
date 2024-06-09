# MC Kinode Interface
_Minecraft Java Plugin Template using Maven and PaperMC API_


## Features

> _Manager_ is a code pattern that I use to manage the plugin and its features.

- **Manager**: _Manager_ is a class that handles a specific feature of the plugin.
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

# Use Managers

1. Create a new class that extends the Manager class
```java
public class MyManager extends Manager {
    public MyManager(Plugin plugin) {
        super(plugin);
    }
    
    @Override
    public void register() {
        // Register the manager task(s)
    }
}
```

2. Create a new instance of the manager in PluginMgr class (Main Manager)
```java
public class PluginMgr extends Manager {
    private MyManager myManager;
    
    public PluginMgr(Plugin plugin) {
        super(plugin);
    }
    
    @Override
    public void register() {
        myManager = new MyManager();
    }
    
    public MyManager getMyManager() {
        return myManager;
    }
    ..
}
```

3. And use it anywhere you want

```java
import net.example.PluginInstance;

public class MyListener implements Listener {

    @EventHandler
    public void onPlayerJoin(PlayerJoinEvent event) {
        Player player = event.getPlayer();
        MyManager myManager = PluginInstance.getInstance().getPluginMgr().getMyManager();
        
        if (myManager.isMyFeatureEnabled())
            player.getInventory().addItem(ItemsUtils.createItem(Material.DIAMOND, 1, "My Item"));
    }
}
```

## Use ItemsUtils

Create an ItemStack with a name and a lore
```java
ItemStack item = Items.createItem(Material.DIAMOND, 1, "My Item", Arrays.asList("My Lore"));
```

Create an Skull item with a name and a lore
```java
ItemStack item = Items.createSkull(Material.DIAMOND, 1, "My Item", Arrays.asList("My Lore"), "Notch");
```

## Listeners

Register your listener in the PluginMgr class inside the registerEvents() method
```java
public class PluginMgr extends Manager {
    
    ..
    private void registerEvents() {
        PluginInstance.getInstance().getServer().getPluginManager()
                .registerEvents(new MyListener(), PluginInstance.getInstance());
    }
    ..
}
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

