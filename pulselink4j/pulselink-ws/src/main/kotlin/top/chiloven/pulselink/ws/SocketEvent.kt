package top.chiloven.pulselink.ws

/** Socket lifecycle/control event. */
public sealed interface SocketEvent {

    public data class StateChanged(public val state: SocketState) : SocketEvent

    public data class ClientAttached(public val clientId: String) : SocketEvent

    public data class ClientDisconnected(public val clientId: String) : SocketEvent

    public data object Heartbeat : SocketEvent

    public data class Pong(public val ts: Long?) : SocketEvent

    public data object IdleTimeout : SocketEvent

    public data class Closed(public val code: Int?, public val reason: String) : SocketEvent

    public data class Error(public val message: String) : SocketEvent

}
