package top.chiloven.pulselink.protocol.websocket.v4

import kotlinx.serialization.KSerializer
import kotlinx.serialization.SerializationException
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.descriptors.buildClassSerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import kotlinx.serialization.json.JsonDecoder
import kotlinx.serialization.json.JsonEncoder
import kotlinx.serialization.json.jsonObject

/** Serializer for [AppMessage]. */
public object AppMessageSerializer : KSerializer<AppMessage> {

    override val descriptor: SerialDescriptor = buildClassSerialDescriptor("AppMessage")

    override fun serialize(encoder: Encoder, value: AppMessage) {
        val jsonEncoder = encoder as? JsonEncoder
            ?: throw SerializationException("AppMessage requires a JSON encoder")
        jsonEncoder.encodeJsonElement(AppMessage.toJson(value))
    }

    override fun deserialize(decoder: Decoder): AppMessage {
        val jsonDecoder = decoder as? JsonDecoder
            ?: throw SerializationException("AppMessage requires a JSON decoder")
        return AppMessage.fromJson(jsonDecoder.decodeJsonElement().jsonObject)
    }

}
