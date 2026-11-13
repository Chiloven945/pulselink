package top.chiloven.pulselink.protocol.coyote.v2

import top.chiloven.pulselink.protocol.ProtocolException

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
