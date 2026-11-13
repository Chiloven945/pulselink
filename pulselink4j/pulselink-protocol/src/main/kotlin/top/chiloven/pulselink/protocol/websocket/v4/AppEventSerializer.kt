package top.chiloven.pulselink.protocol.websocket.v4

import top.chiloven.pulselink.protocol.ProtocolException
import kotlinx.serialization.KSerializer
import kotlinx.serialization.SerializationException
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.descriptors.buildClassSerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import kotlinx.serialization.json.JsonDecoder
import kotlinx.serialization.json.JsonEncoder
import kotlinx.serialization.json.JsonObject

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
