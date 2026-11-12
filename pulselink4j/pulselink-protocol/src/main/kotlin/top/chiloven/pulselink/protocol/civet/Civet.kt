package top.chiloven.pulselink.protocol.civet

import top.chiloven.pulselink.core.Pressure
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
public const val DEVICE_NAME: String = "47L124000"

/** Start or stop Civet pressure reporting. */
public data class Command50(
    public val color: Int,
    public val start: Boolean,
) {

    /** Encodes the 17-byte command. */
    public fun encode(): ByteArray =
        ByteArray(17).also {
            it[0] = 0x50.toU8()
            it[1] = color.toU8()
            it[2] = (if (start) 0xD0 else 0x00).toU8()
        }

}

/** Reset pressure and/or set display orientation. */
public data class Command66(
    public val orientation: Int? = null,
    public val reset: Boolean = false,
) {

    /** Encodes the 13-byte command. */
    public fun encode(): ByteArray =
        when {
            orientation != null && orientation !in listOf(1, 3) ->
                throw ProtocolException.ValueOutOfRange(
                    "Civet orientation",
                    orientation.toLong(),
                    1,
                    3
                )

            else ->
                ByteArray(13).also {
                    it[0] = 0x66.toU8()
                    it[10] = (orientation ?: 0).toU8()
                    it[12] = (if (reset) 2 else 0).toU8()
                }
        }

}

/** A Civet pressure notification. */
public data class D0Message(
    public val color: Int,
    public val pressure: Pressure,
) {

    public companion object {

        /** Decodes a 17-byte pressure notification. */
        public fun decode(bytes: ByteArray): D0Message {
            bytes.requireLength(17)
            if (bytes.u8(0) != 0xD0)
                throw ProtocolException.InvalidHeader(bytes.u8(0))

            val raw = (bytes.u8(8) or (bytes.u8(9) shl 8)).toShort().toInt()
            return D0Message(
                bytes.u8(1),
                Pressure.fromCentiKpa(raw)
            )
        }

    }

}
