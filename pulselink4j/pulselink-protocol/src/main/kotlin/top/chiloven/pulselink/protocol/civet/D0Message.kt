package top.chiloven.pulselink.protocol.civet

import top.chiloven.pulselink.core.Pressure
import top.chiloven.pulselink.protocol.ProtocolException
import top.chiloven.pulselink.protocol.requireLength
import top.chiloven.pulselink.protocol.u8

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
