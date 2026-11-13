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

/** Serializer for [RelayFrame]. */
public object RelayFrameSerializer : KSerializer<RelayFrame> {

    override val descriptor: SerialDescriptor = buildClassSerialDescriptor("RelayFrame")

    override fun serialize(encoder: Encoder, value: RelayFrame) {
        val jsonEncoder = encoder as? JsonEncoder
            ?: throw SerializationException("RelayFrame requires a JSON encoder")
        jsonEncoder.encodeJsonElement(RelayFrame.toJson(value))
    }

    override fun deserialize(decoder: Decoder): RelayFrame {
        val jsonDecoder = decoder as? JsonDecoder
            ?: throw SerializationException("RelayFrame requires a JSON decoder")
        return RelayFrame.fromJson(jsonDecoder.decodeJsonElement().jsonObject)
    }

}
