package top.chiloven.pulselink.ws

/** Typed remote snapshot wrapper. */
public sealed interface TypedRemoteSnapshot {

    public val snapshot: RemoteDeviceSnapshot

    public data class Coyote(override val snapshot: RemoteDeviceSnapshot) : TypedRemoteSnapshot

    public data class Civet(override val snapshot: RemoteDeviceSnapshot) : TypedRemoteSnapshot

    public data class Opossum(override val snapshot: RemoteDeviceSnapshot) : TypedRemoteSnapshot

    public data class Unknown(override val snapshot: RemoteDeviceSnapshot) : TypedRemoteSnapshot

}
