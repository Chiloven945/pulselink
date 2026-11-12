package top.chiloven.pulselink.protocol.websocket.v4

import kotlinx.serialization.KSerializer
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.SerializationException
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonDecoder
import kotlinx.serialization.json.JsonEncoder
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.intOrNull
import kotlinx.serialization.json.jsonPrimitive
import top.chiloven.pulselink.protocol.ProtocolException

/** Supported V4 device operation types. */
public enum class ActionType(public val wireValue: Int) {

    AppendPulseData(0),
    AddIntensity(3),
    SetTempIntensity(4),
    SetIntensity(7);

    public companion object {

        public fun fromWire(value: Int): ActionType =
            entries.firstOrNull { it.wireValue == value }
                ?: throw ProtocolException.UnsupportedActionType(value)

    }

}

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

/** A pulse frame in hexadecimal or numeric form. */
public sealed interface PulseFrameWire {

    public data class Hex(public val value: String) : PulseFrameWire

    public data class Numbers(public val values: List<Int>) : PulseFrameWire

    /** Validates the byte length and hex encoding. */
    public fun validate(expectedBytes: Int) {
        when (this) {
            is Hex -> when {
                value.length != expectedBytes * 2 ->
                    throw ProtocolException.InvalidPulseFrame("hex frame has the wrong byte length")

                !value.all { it in '0'..'9' || it in 'a'..'f' || it in 'A'..'F' } ->
                    throw ProtocolException.InvalidPulseFrame("hex frame contains a non-hex character")
            }

            is Numbers -> when {
                values.size != expectedBytes ->
                    throw ProtocolException.InvalidPulseFrame("numeric frame has the wrong length")

                values.any { it !in 0..255 } ->
                    throw ProtocolException.InvalidPulseFrame("numeric frame contains a non-byte value")
            }

        }
    }

}

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

/** The `v` field of a device operation. */
public sealed interface OperateValue {

    public data class Intensity(public val value: Int) : OperateValue

    public data class PulseFrames(public val frames: List<PulseFrameWire>) : OperateValue

}

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

/** A V4 `device.op` request payload. */
@Serializable
public data class DeviceOperate(
    @SerialName("s") public val slotId: String,
    @SerialName("t") @Serializable(with = ActionTypeSerializer::class) public val action: ActionType,
    @SerialName("c") public val channel: Int,
    @SerialName("p") public val priority: Int? = null,
    @SerialName("d") public val durationMs: Long? = null,
    @SerialName("v") @Serializable(with = OperateValueSerializer::class) public val value: OperateValue,
    @SerialName("im") public val immediate: Boolean? = null,
    @SerialName("ver") public val ver: Int? = null,
    @SerialName("seq") public val seq: Long? = null,
) {

    /** Validates cross-field constraints that serde cannot express. */
    public fun validate(): DeviceOperate {
        if (slotId.isEmpty()) {
            throw ProtocolException.InvalidParameters("slotId must not be empty")
        }

        if (channel !in 0..1) {
            throw ProtocolException.ValueOutOfRange(
                "device channel",
                channel.toLong(),
                0,
                1
            )
        }

        priority?.let {
            if (it !in 0..2) throw ProtocolException.ValueOutOfRange(
                "device operation priority",
                it.toLong(),
                0,
                2
            )
        }

        when (action) {
            ActionType.AppendPulseData -> validatePulseFrames()
            ActionType.AddIntensity -> {
                val v = (value as? OperateValue.Intensity)?.value
                    ?: throw ProtocolException.InvalidParameters("AddIntensity requires an integer value")
                if (v !in -200..200) throw ProtocolException.ValueOutOfRange(
                    "AddIntensity value",
                    v.toLong(),
                    -200,
                    200
                )
            }

            ActionType.SetTempIntensity -> {
                val v = (value as? OperateValue.Intensity)?.value
                    ?: throw ProtocolException.InvalidParameters("SetTempIntensity requires an integer value")
                if (v !in 0..200) throw ProtocolException.ValueOutOfRange(
                    "SetTempIntensity value",
                    v.toLong(),
                    0,
                    200
                )
            }

            ActionType.SetIntensity -> {
                val v = (value as? OperateValue.Intensity)?.value
                    ?: throw ProtocolException.InvalidParameters("SetIntensity requires an integer value")
                if (v != 0) throw ProtocolException.InvalidParameters("SetIntensity accepts only value 0")
            }
        }
        return this
    }

    private fun validatePulseFrames() {
        val version = ver ?: 3
        if (version !in 2..3)
            throw ProtocolException.ValueOutOfRange(
                "pulse frame version",
                version.toLong(),
                2,
                3
            )
        val expected = if (version == 2) 3 else 8
        val frames = (value as? OperateValue.PulseFrames)?.frames
            ?: throw ProtocolException.InvalidParameters("AppendPulseData requires pulse frames")
        if (frames.isEmpty())
            throw ProtocolException.InvalidPulseFrame("AppendPulseData requires at least one frame")
        frames.forEach { it.validate(expected) }
    }

}

/** A V4 `device.op.clear` request payload. */
@Serializable
public data class ClearOperate(
    @SerialName("s") public val slotId: String? = null,
    @SerialName("c") public val channel: Int? = null,
) {

    /** Validates that a channel is not specified without a slot id. */
    public fun validate(): ClearOperate {
        channel?.let {
            when {
                slotId == null -> throw ProtocolException.InvalidParameters(
                    "device.op.clear channel requires slotId"
                )

                it !in 0..1 -> throw ProtocolException.ValueOutOfRange(
                    "clear channel",
                    it.toLong(),
                    0,
                    1
                )
            }
        }
        return this
    }

}

/** Why a device operation ended. */
@Serializable
public enum class OperationEndReason {

    @SerialName("completed")
    Completed,

    @SerialName("cleared")
    Cleared,

    @SerialName("replaced")
    Replaced,

    @SerialName("cancelled")
    Cancelled,

}

/** The result of a completed `device.op` request. */
@Serializable
public data class DeviceOperateResult(
    @SerialName("type") @Serializable(with = ActionTypeSerializer::class) public val action: ActionType,
    public val reason: OperationEndReason,
    @SerialName("slotId") public val slotId: String? = null,
    public val channel: Int? = null,
)
