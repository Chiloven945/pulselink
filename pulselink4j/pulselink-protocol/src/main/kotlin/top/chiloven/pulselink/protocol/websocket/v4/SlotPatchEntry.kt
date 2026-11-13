package top.chiloven.pulselink.protocol.websocket.v4

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonObject

/** One changed slot in a `slots.patch` event. */
@Serializable
public data class SlotPatchEntry(
    @SerialName("slotId") public val slotId: String,
    public val props: JsonObject = JsonObject(emptyMap()),
    @SerialName("slotState") public val slotState: JsonObject = JsonObject(emptyMap()),
)
