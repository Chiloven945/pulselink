package top.chiloven.pulselink.protocol.websocket.v4

import top.chiloven.pulselink.protocol.ProtocolException
import kotlinx.serialization.KSerializer
import kotlinx.serialization.SerializationException
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import kotlinx.serialization.json.*

/** Serializer for [OperateValue] using the untagged V4 representation. */
public object OperateValueSerializer : KSerializer<OperateValue> {

    override val descriptor: SerialDescriptor = PrimitiveSerialDescriptor(
        "OperateValue",
        PrimitiveKind.STRING
    )

    override fun serialize(encoder: Encoder, value: OperateValue) {
        val jsonEncoder = encoder as? JsonEncoder
            ?: throw SerializationException("OperateValue requires a JSON encoder")
        jsonEncoder.encodeJsonElement(
            when (value) {
                is OperateValue.Intensity -> JsonPrimitive(value.value)
                is OperateValue.PulseFrames -> buildJsonArray {
                    value.frames.forEach {
                        add(
                            PulselinkJson.encodeToJsonElement(
                                PulseFrameWireSerializer,
                                it
                            )
                        )
                    }
                }
            },
        )
    }

    override fun deserialize(decoder: Decoder): OperateValue {
        val jsonDecoder = decoder as? JsonDecoder
            ?: throw SerializationException("OperateValue requires a JSON decoder")
        return when (val element = jsonDecoder.decodeJsonElement()) {
            is JsonPrimitive -> {
                val value = element.intOrNull
                    ?: throw ProtocolException.InvalidParameters("device operation v must be an integer or frame array")
                OperateValue.Intensity(value)
            }

            is JsonArray -> OperateValue.PulseFrames(
                element.map { PulselinkJson.decodeFromJsonElement(PulseFrameWireSerializer, it) },
            )

            else -> throw ProtocolException.InvalidParameters("device operation v must be an integer or frame array")
        }
    }

}
