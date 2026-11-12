package top.chiloven.pulselink.protocol.websocket.v4

import top.chiloven.pulselink.protocol.ProtocolException
import kotlinx.serialization.KSerializer
import kotlinx.serialization.SerializationException
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.descriptors.buildClassSerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import kotlinx.serialization.json.*

/** An event sent by a V4 app client. */
public sealed interface AppEvent {

    public data class DevicesSnapshot(public val devices: List<RemoteDevice>) : AppEvent

    public data class DevicesPatch(
        public val added: List<RemoteDevice>,
        public val removed: List<String>,
    ) : AppEvent

    public data class SlotsPatch(public val slots: List<SlotPatchEntry>) : AppEvent

    public data class CustomAction(public val action: Int) : AppEvent

    /** An event type unknown to this SDK version, preserving the original object. */
    public data class Unknown(public val raw: JsonObject) : AppEvent

    /** Validates event-specific constraints. */
    public fun validate(): AppEvent {
        if (this is CustomAction && action !in 0..9) {
            throw ProtocolException.ValueOutOfRange("custom action", action.toLong(), 0, 9)
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
            when (val type = raw["ev"]?.jsonPrimitive?.contentOrNull) {
                "devices.snapshot" -> DevicesSnapshot(
                    PulselinkJson.decodeFromJsonElement(raw.getValue("devices")),
                )

                "devices.patch" -> DevicesPatch(
                    added = raw["added"]?.let { PulselinkJson.decodeFromJsonElement(it) }
                        ?: emptyList(),
                    removed = raw["removed"]?.let { PulselinkJson.decodeFromJsonElement(it) }
                        ?: emptyList(),
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

private val JsonPrimitive.contentOrNull: String?
    get() = if (isString) content else null

/** Serializer for [AppEvent]. */
public object AppEventSerializer : KSerializer<AppEvent> {

    override val descriptor: SerialDescriptor = buildClassSerialDescriptor("AppEvent")

    override fun serialize(encoder: Encoder, value: AppEvent) {
        val jsonEncoder = encoder as? JsonEncoder
            ?: throw SerializationException("AppEvent requires a JSON encoder")
        jsonEncoder.encodeJsonElement(AppEvent.toJson(value))
    }

    override fun deserialize(decoder: Decoder): AppEvent {
        val jsonDecoder = decoder as? JsonDecoder
            ?: throw SerializationException("AppEvent requires a JSON decoder")
        val element = jsonDecoder.decodeJsonElement()
        val objectValue = element as? JsonObject
            ?: throw ProtocolException.Json("app event must be a JSON object")
        return AppEvent.fromJson(objectValue)
    }

}

/** Encodes an [AppEvent] as V4 JSON. */
public fun encodeAppEvent(event: AppEvent): String =
    PulselinkJson.encodeToString(AppEventSerializer, event)

/** Decodes an [AppEvent] from V4 JSON. */
public fun decodeAppEvent(json: String): AppEvent =
    PulselinkJson.decodeFromString(AppEventSerializer, json)
