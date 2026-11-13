package top.chiloven.pulselink.ws

/** Socket lifecycle state. */
public enum class SocketState {

    Connecting,
    WaitingForPeer,
    Ready,
    Closing,
    Closed,

}
