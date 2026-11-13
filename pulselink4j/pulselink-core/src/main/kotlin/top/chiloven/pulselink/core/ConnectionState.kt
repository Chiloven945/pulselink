package top.chiloven.pulselink.core

/**
 * The lifecycle state of a device session.
 */
public enum class ConnectionState {

    Disconnected,
    Connecting,
    Initializing,
    Ready,
    Degraded,
    Reconnecting,
    Closing,

}
