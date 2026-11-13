package top.chiloven.pulselink.java.ws;

/** Socket lifecycle state. */
public enum SocketState {

    CONNECTING,
    WAITING_FOR_PEER,
    READY,
    CLOSING,
    CLOSED

}
