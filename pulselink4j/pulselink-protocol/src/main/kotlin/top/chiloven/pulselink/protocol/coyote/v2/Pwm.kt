package top.chiloven.pulselink.protocol.coyote.v2

import top.chiloven.pulselink.protocol.ProtocolException
import top.chiloven.pulselink.protocol.requireLength
import top.chiloven.pulselink.protocol.u8

/** A Coyote V2 wire intensity in the range `0..=2047`. */
@JvmInline
public value class CoyoteV2Intensity private constructor(
    public val value: Int
) : Comparable<CoyoteV2Intensity> {

    override fun compareTo(other: CoyoteV2Intensity): Int =
        value.compareTo(other.value)

    /** Returns the approximate official-app display value (`0..=200`). */
    public fun display(): Int =
        (value / 7).coerceAtMost(200)

    public companion object {

        public val MIN: CoyoteV2Intensity = CoyoteV2Intensity(0)
        public val MAX: CoyoteV2Intensity = CoyoteV2Intensity(2047)

        /** Creates a wire intensity. */
        public fun of(value: Int): CoyoteV2Intensity =
            when (value) {
                in 0..2047 -> CoyoteV2Intensity(value)
                else -> throw ProtocolException.ValueOutOfRange(
                    "Coyote V2 intensity",
                    value.toLong(),
                    0,
                    2047
                )
            }

        /** Creates an intensity from the official display scale (`0..=200`). */
        public fun fromDisplay(value: Int): CoyoteV2Intensity =
            when (value) {
                in 0..200 -> CoyoteV2Intensity(value * 7)
                else -> throw ProtocolException.ValueOutOfRange(
                    "Coyote V2 display intensity",
                    value.toLong(),
                    0,
                    200
                )
            }

    }

}

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
