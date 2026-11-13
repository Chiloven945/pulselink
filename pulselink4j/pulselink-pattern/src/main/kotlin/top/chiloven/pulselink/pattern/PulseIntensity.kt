package top.chiloven.pulselink.pattern

/**
 * A relative pulse intensity in the range `0..=100`.
 */
@JvmInline
public value class PulseIntensity private constructor(
    public val value: Int
) : Comparable<PulseIntensity> {

    override fun compareTo(other: PulseIntensity): Int =
        value.compareTo(other.value)

    override fun toString(): String =
        value.toString()

    public companion object {

        public val MIN: PulseIntensity = PulseIntensity(0)
        public val MAX: PulseIntensity = PulseIntensity(100)

        public fun of(value: Int): PulseIntensity =
            when (value) {
                in 0..100 -> PulseIntensity(value)
                else -> throw PatternException.InvalidPulseIntensity(value)
            }

    }

}
