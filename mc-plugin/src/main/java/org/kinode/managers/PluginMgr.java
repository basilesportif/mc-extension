package org.kinode.managers;

public class PluginMgr extends Manager {

    /**
     * @param registerAtStartUp If true, the manager will be registered at startup
     *                          PluginMgr is the main manager, it handles other
     *                          managers and plugins tasks.
     */
    public PluginMgr(boolean... registerAtStartUp) {
        super(registerAtStartUp);
    }

    /**
     * Register the plugins tasks (Other managers, events, commands, etc.)
     */
    @Override
    public void register() {
        if (initialized) // Don't register if already registered
            return;
        registerManagers();
        registerEvents();
        registerCommands();
    }

    private ConfigMgr configMgr;

    // Register other managers: Managers are classes that handle a specific task
    private void registerManagers() {
        configMgr = new ConfigMgr();
    }

    // Listeners
    private void registerEvents() {
        // PluginInstance.getInstance().getServer().getPluginManager()
        // .registerEvents(new MyListener(), PluginInstance.getInstance());
    }

    // Commands
    private void registerCommands() {
        // PluginInstance.getInstance().getCommand("mycommand").setExecutor(new
        // MyCommand()
    }

    // Manager Getters

    public ConfigMgr getConfigMgr() {
        return configMgr;
    }
}
