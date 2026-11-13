package top.chiloven.pulselink.protocol.coyote.v2

import top.chiloven.pulselink.protocol.ProtocolException
import top.chiloven.pulselink.protocol.requireLength
import top.chiloven.pulselink.protocol.u8

/** A V2 pulse parameter set `[X, Y, Z]`. */
public data class PwmPulse(
    public val x: Int,
    public val y: Int,
    public val z: Int,
) {

    init {
        when {
            x !in 0..31 -> throw ProtocolException.ValueOutOfRange(
                "Coyote V2 X",
                x.toLong(),
                0,
                31
            )

            y !in 0..1023 -> throw ProtocolException.ValueOutOfRange(
                "Coyote V2 Y",
                y.toLong(),
                0,
                1023
            )

            z !in 0..31 -> throw ProtocolException.ValueOutOfRange(
                "Coyote V2 Z",
                z.toLong(),
                0,
                31
            )
        }
    }

    /** Encodes the three-byte little-endian value. */
    public fun encode(): ByteArray =
        (x or (y shl 5) or (z shl 15)).let { raw ->
            byteArrayOf(
                (raw and 0xFF).toByte(),
                ((raw ushr 8) and 0xFF).toByte(),
                ((raw ushr 16) and 0xFF).toByte(),
            )
        }

    public companion object {

        /** Decodes a three-byte value. */
        public fun decode(bytes: ByteArray): PwmPulse {
            bytes.requireLength(3)
            val raw = bytes.u8(0) or
                    (bytes.u8(1) shl 8) or
                    (bytes.u8(2) shl 16)
            return PwmPulse(
                x = raw and 0x1F,
                y = (raw ushr 5) and 0x3FF,
                z = (raw ushr 15) and 0x1F,
            )
        }

    }

}
