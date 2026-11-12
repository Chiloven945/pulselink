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

/** Incremental device list patch. */
@Serializable
public data class DevicePatch(
    public val added: List<RemoteDevice> = emptyList(),
    public val removed: List<String> = emptyList(),
)

/** Incremental slot state patch. */
@Serializable
public data class SlotPatch(
    public val slots: List<SlotPatchEntry> = emptyList(),
)

/** One changed slot in a `slots.patch` event. */
@Serializable
public data class SlotPatchEntry(
    @SerialName("slotId") public val slotId: String,
    public val props: JsonObject = JsonObject(emptyMap()),
    @SerialName("slotState") public val slotState: JsonObject = JsonObject(emptyMap()),
)
