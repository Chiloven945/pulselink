package top.chiloven.pulselink.java.core;

/** The lifecycle state of a device session. */
public enum ConnectionState {

    DISCONNECTED,
    CONNECTING,
    INITIALIZING,
    READY,
    DEGRADED,
    RECONNECTING,
    CLOSING

}
