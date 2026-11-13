package top.chiloven.pulselink.protocol.websocket.v4

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonObject

/** A device exposed by a DG-LAB V4 app client. */
@Serializable
public data class RemoteDevice(
    @SerialName("id") public val id: Int? = null,
    @SerialName("slotId") public val slotId: String,
    public val name: String,
    @SerialName("type") public val deviceType: String,
    public val props: JsonObject = JsonObject(emptyMap()),
    @SerialName("slotState") public val slotState: JsonObject = JsonObject(emptyMap()),
)
