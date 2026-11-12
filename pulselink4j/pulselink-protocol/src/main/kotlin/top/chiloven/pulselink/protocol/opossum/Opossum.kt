package top.chiloven.pulselink.protocol.opossum

import top.chiloven.pulselink.core.ButtonMask
import top.chiloven.pulselink.protocol.ProtocolException
import top.chiloven.pulselink.protocol.requireLength
import top.chiloven.pulselink.protocol.toU8
import top.chiloven.pulselink.protocol.u8
import java.util.*

public val SERVICE_CONTROL: UUID = UUID.fromString("0000180c-0000-1000-8000-00805f9b34fb")
public val CHAR_WRITE: UUID = UUID.fromString("0000150a-0000-1000-8000-00805f9b34fb")
public val CHAR_NOTIFY: UUID = UUID.fromString("0000150b-0000-1000-8000-00805f9b34fb")
public val SERVICE_BATTERY: UUID = UUID.fromString("0000180a-0000-1000-8000-00805f9b34fb")
public val CHAR_BATTERY: UUID = UUID.fromString("00001500-0000-1000-8000-00805f9b34fb")
public const val DEVICE_NAME: String = "47L127000"

private val B2_FIXED: IntArray = intArrayOf(
    0xFF, 0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x08, 0x09,
)

/** Indicator color and button-reporting command (`0x50`). */
public data class Command50(
    public val color: Int,
    public val buttonReport: Boolean,
) {

    public fun encode(): ByteArray =
        byteArrayOf(
            0x50.toU8(),
            color.toU8(),
            (if (buttonReport) 1 else 0).toU8()
        )

}

/** Dual-channel vibration frame (`0xB0`). */
public data class B0(
    public val a: List<Int>,
    public val b: List<Int>,
) {

    init {
        when {
            a.size != 4 || b.size != 4 ->
                throw ProtocolException.InvalidParameters("Opossum B0 requires 4 values per channel")

            (a + b).any { it !in 0..100 } ->
                throw ProtocolException.ValueOutOfRange(
                    "Opossum vibration intensity",
                    101,
                    0,
                    100
                )
        }
    }

    public fun encode(): ByteArray =
        ByteArray(20).also {
            it[0] = 0xB0.toU8()
            a.forEachIndexed { index, value -> it[8 + index] = value.toU8() }
            b.forEachIndexed { index, value -> it[16 + index] = value.toU8() }
        }

}

/** Channel intensity command (`0xB3`); null leaves a channel unchanged. */
public data class B3(
    public val a: Int?,
    public val b: Int?,
) {

    public fun encode(): ByteArray =
        byteArrayOf(0xB3.toU8(), encoded(a), encoded(b))

    private fun encoded(value: Int?): Byte =
        when (value) {
            null -> 0xFF.toU8()

            !in 0..200 -> throw ProtocolException.ValueOutOfRange(
                "Opossum intensity",
                value.toLong(),
                0,
                200
            )

            else -> value.toU8()
        }

}

/** Display intensity sync command (`0xB2`). */
public data class B2(
    public val a: Int,
    public val b: Int,
) {

    public fun encode(): ByteArray =
        when {
            a !in 0..200 || b !in 0..200 -> {
                throw ProtocolException.ValueOutOfRange(
                    "Opossum display intensity",
                    maxOf(a, b).toLong(),
                    0,
                    200
                )
            }

            else -> ByteArray(24).also {
                it[0] = 0xB2.toU8()
                B2_FIXED.forEachIndexed { index, value -> it[1 + index] = value.toU8() }
                it[22] = a.toU8()
                it[23] = b.toU8()
            }
        }

}

public data class B3Message(
    public val a: Int,
    public val b: Int
) {

    public companion object {

        public fun decode(bytes: ByteArray): B3Message {
            bytes.requireLength(3)
            if (bytes.u8(0) != 0xB3) {
                throw ProtocolException.InvalidHeader(bytes.u8(0))
            }

            val a = bytes.u8(1)
            val b = bytes.u8(2)
            return when {
                a !in 0..200 || b !in 0..200 ->
                    throw ProtocolException.ValueOutOfRange(
                        "Opossum intensity notification",
                        maxOf(a, b).toLong(),
                        0,
                        200
                    )

                else -> B3Message(a, b)
            }

        }

    }
}

public data class D0Message(
    public val sequence: Int,
    public val buttons: ButtonMask,
    public val raw: ByteArray,
) {

    public companion object {

        public fun decode(bytes: ByteArray): D0Message {
            bytes.requireLength(16)
            if (bytes.u8(0) != 0xD0) {
                throw ProtocolException.InvalidHeader(bytes.u8(0))
            }

            val bits = bytes.u8(2) or (bytes.u8(3) shl 8)
            return D0Message(
                bytes.u8(1),
                ButtonMask.fromBits(bits),
                bytes.copyOf()
            )
        }

    }

}

/** A decoded Opossum notification. */
public sealed interface Message {

    public data class B3Value(public val message: B3Message) : Message
    public data class D0Value(public val message: D0Message) : Message

    public companion object {

        public fun decode(bytes: ByteArray): Message =
            when (bytes.firstOrNull()?.toInt()?.and(0xFF)) {
                0xB3 -> B3Value(B3Message.decode(bytes))
                0xD0 -> D0Value(D0Message.decode(bytes))
                null -> throw ProtocolException.InvalidLength(3, 0)
                else -> throw ProtocolException.InvalidHeader(bytes.u8(0))
            }

    }

}
