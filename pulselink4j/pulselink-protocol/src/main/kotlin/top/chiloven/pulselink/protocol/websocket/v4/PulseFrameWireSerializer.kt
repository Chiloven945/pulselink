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

/** Serializer for [PulseFrameWire]. */
public object PulseFrameWireSerializer : KSerializer<PulseFrameWire> {

    override val descriptor: SerialDescriptor = PrimitiveSerialDescriptor(
        "PulseFrameWire",
        PrimitiveKind.STRING
    )

    override fun serialize(encoder: Encoder, value: PulseFrameWire) {
        val jsonEncoder = encoder as? JsonEncoder
            ?: throw SerializationException("PulseFrameWire requires a JSON encoder")
        jsonEncoder.encodeJsonElement(
            when (value) {
                is PulseFrameWire.Hex -> JsonPrimitive(value.value)
                is PulseFrameWire.Numbers -> buildJsonArray {
                    value.values.forEach {
                        add(
                            JsonPrimitive(it)
                        )
                    }
                }
            },
        )
    }

    override fun deserialize(decoder: Decoder): PulseFrameWire {
        val jsonDecoder = decoder as? JsonDecoder
            ?: throw SerializationException("PulseFrameWire requires a JSON decoder")
        return when (val element = jsonDecoder.decodeJsonElement()) {
            is JsonPrimitive -> if (element.isString) {
                PulseFrameWire.Hex(element.content)
            } else {
                throw ProtocolException.InvalidPulseFrame("pulse frame must be hex or a byte array")
            }

            is JsonArray -> PulseFrameWire.Numbers(
                element.map {
                    it.jsonPrimitive.intOrNull
                        ?: throw ProtocolException.InvalidPulseFrame("numeric frame contains a non-integer")
                },
            )

            else -> throw ProtocolException.InvalidPulseFrame("pulse frame must be hex or a byte array")
        }
    }

}
