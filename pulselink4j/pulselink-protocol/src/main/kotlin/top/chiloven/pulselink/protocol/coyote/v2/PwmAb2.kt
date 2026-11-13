package top.chiloven.pulselink.protocol.coyote.v2

import top.chiloven.pulselink.protocol.requireLength
import top.chiloven.pulselink.protocol.u8

/** A/B channel intensity (`PWM_AB2`). */
public data class PwmAb2(
    public val a: CoyoteV2Intensity,
    public val b: CoyoteV2Intensity,
) {

    /** Encodes the three-byte little-endian value. */
    public fun encode(): ByteArray =
        ((a.value shl 11) or b.value).let {
            byteArrayOf(
                (it and 0xFF).toByte(),
                ((it ushr 8) and 0xFF).toByte(),
                ((it ushr 16) and 0xFF).toByte(),
            )
        }

    public companion object {

        /** Decodes a three-byte value. */
        public fun decode(bytes: ByteArray): PwmAb2 {
            bytes.requireLength(3)
            val raw = bytes.u8(0) or
                    (bytes.u8(1) shl 8) or
                    (bytes.u8(2) shl 16)
            return PwmAb2(
                CoyoteV2Intensity.of((raw ushr 11) and 0x7FF),
                CoyoteV2Intensity.of(raw and 0x7FF),
            )
        }

    }

}
