package top.chiloven.pulselink.protocol.websocket.v4

import kotlinx.serialization.KSerializer
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder

/** Serializer for [ActionType] using its numeric wire value. */
public object ActionTypeSerializer : KSerializer<ActionType> {

    override val descriptor: SerialDescriptor = PrimitiveSerialDescriptor(
        "ActionType",
        PrimitiveKind.INT
    )

    override fun serialize(encoder: Encoder, value: ActionType): Unit =
        encoder.encodeInt(value.wireValue)

    override fun deserialize(decoder: Decoder): ActionType =
        ActionType.fromWire(decoder.decodeInt())

}
