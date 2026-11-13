package top.chiloven.pulselink.protocol.websocket.v4

import top.chiloven.pulselink.protocol.ProtocolException
import top.chiloven.pulselink.protocol.websocket.v4.internal.stringContentOrNull
import top.chiloven.pulselink.protocol.websocket.v4.internal.valueAs
import kotlinx.serialization.json.*

/** An event sent by a V4 app client. */
public sealed interface AppEvent {

    public data class DevicesSnapshot(
        public val devices: List<RemoteDevice>
    ) : AppEvent

    public data class DevicesPatch(
        public val added: List<RemoteDevice>,
        public val removed: List<String>,
    ) : AppEvent

    public data class SlotsPatch(
        public val slots: List<SlotPatchEntry>
    ) : AppEvent

    public data class CustomAction(
        public val action: Int
    ) : AppEvent

    /** An event type unknown to this SDK version, preserving the original object. */
    public data class Unknown(
        public val raw: JsonObject
    ) : AppEvent

    /** Validates event-specific constraints. */
    public fun validate(): AppEvent {
        if (this is CustomAction && action !in 0..9) {
            throw ProtocolException.ValueOutOfRange(
                field = "custom action",
                value = action.toLong(),
                min = 0,
                max = 9
            )
        }
        return this
    }

    public companion object {

        /** Encodes the event fields, including the `ev` discriminator. */
        public fun toJson(event: AppEvent): JsonObject =
            buildJsonObject {
                when (event) {
                    is DevicesSnapshot -> {
                        put("ev", "devices.snapshot")
                        put("devices", PulselinkJson.encodeToJsonElement(event.devices))
                    }

                    is DevicesPatch -> {
                        put("ev", "devices.patch")
                        put("added", PulselinkJson.encodeToJsonElement(event.added))
                        put("removed", PulselinkJson.encodeToJsonElement(event.removed))
                    }

                    is SlotsPatch -> {
                        put("ev", "slots.patch")
                        put("slots", PulselinkJson.encodeToJsonElement(event.slots))
                    }

                    is CustomAction -> {
                        put("ev", "custom.action")
                        put("action", event.action)
                    }

                    is Unknown -> return event.raw
                }
            }

        /** Decodes an event object. */
        public fun fromJson(raw: JsonObject): AppEvent =
            when (val type = raw["ev"]?.jsonPrimitive?.stringContentOrNull) {
                "devices.snapshot" -> DevicesSnapshot(
                    PulselinkJson.decodeFromJsonElement(raw.getValue("devices")),
                )

                "devices.patch" -> DevicesPatch(
                    added = raw.valueAs("added") ?: emptyList(),
                    removed = raw.valueAs("removed") ?: emptyList(),
                )

                "slots.patch" -> SlotsPatch(
                    PulselinkJson.decodeFromJsonElement(raw.getValue("slots")),
                )

                "custom.action" -> CustomAction(
                    raw["action"]?.jsonPrimitive?.intOrNull
                        ?: throw ProtocolException.Json("custom.action requires an integer action"),
                )

                null -> throw ProtocolException.Json("event has no ev discriminator")
                else -> Unknown(raw)
            }

    }

}
