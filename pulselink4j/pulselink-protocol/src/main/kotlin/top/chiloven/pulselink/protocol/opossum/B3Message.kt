package top.chiloven.pulselink.protocol.opossum

import top.chiloven.pulselink.protocol.ProtocolException
import top.chiloven.pulselink.protocol.requireLength
import top.chiloven.pulselink.protocol.u8

/** An Opossum `0xB3` intensity notification. */
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
