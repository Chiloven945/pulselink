package top.chiloven.pulselink.core

/**
 * A battery percentage in the range `0..=100`.
 */
@JvmInline
public value class BatteryLevel private constructor(
    public val value: Int
) : Comparable<BatteryLevel> {

    override fun compareTo(other: BatteryLevel): Int =
        value.compareTo(other.value)

    override fun toString(): String =
        "$value%"

    public companion object {

        public val MIN: BatteryLevel = BatteryLevel(0)
        public val MAX: BatteryLevel = BatteryLevel(100)

        public fun of(value: Int): BatteryLevel =
            when (value) {
                in 0..100 -> BatteryLevel(value)
                else -> throw CoreException.InvalidBatteryLevel(value)
            }

    }

}

/** Observed battery information. */
public data class BatteryState(public val level: BatteryLevel)
