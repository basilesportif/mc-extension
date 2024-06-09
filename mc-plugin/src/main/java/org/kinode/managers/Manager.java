package org.kinode.managers;

public class Manager {

    protected boolean initialized = false;

    /**
     * @param registerAtStartUp If true, the manager will be registered at startup
     *                          Manager is a class that handles a specific task, use
     *                          it to organize your code.
     *                          Create a new class that extends Manager and override
     *                          the register() method.
     *                          Example: ConfigMgr handles the config.yml file and
     *                          its values
     */
    public Manager(boolean... registerAtStartUp) {
        if (registerAtStartUp.length > 0 && !registerAtStartUp[0])
            return;
        register();

        this.initialized = true;
    }

    /**
     * Register the manager tasks
     */
    public void register() {
        // Don't write code here, override this method in your extended Manager class
    }

    /**
     * @return The Plugin Instance
     */
    public final boolean isInitialized() {
        return this.initialized;
    }

    /**
     * @return The name of the manager class
     */
    public final String name() {
        return getClass().getSimpleName();
    }

    public final String toString() {
        return name();
    }

    public final boolean equals(Object obj) {
        return obj instanceof Manager && ((Manager) obj).name().equals(name());
    }

    public final int hashCode() {
        return name().hashCode();
    }
}
