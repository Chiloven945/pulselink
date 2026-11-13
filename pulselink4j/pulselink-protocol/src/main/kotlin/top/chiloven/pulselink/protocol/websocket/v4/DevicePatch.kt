package top.chiloven.pulselink.protocol.websocket.v4

import kotlinx.serialization.Serializable

/** Incremental device list patch. */
@Serializable
public data class DevicePatch(
    public val added: List<RemoteDevice> = emptyList(),
    public val removed: List<String> = emptyList(),
)
