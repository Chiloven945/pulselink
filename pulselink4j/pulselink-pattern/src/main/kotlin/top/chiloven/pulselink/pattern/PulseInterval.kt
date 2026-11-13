package top.chiloven.pulselink.pattern

import kotlin.math.round
import kotlin.time.Duration
import kotlin.time.Duration.Companion.milliseconds

/**
 * A pulse interval in milliseconds, in the range `10..=1000`.
 */
@JvmInline
public value class PulseInterval private constructor(
    public val milliseconds: Int
) : Comparable<PulseInterval> {

    override fun compareTo(other: PulseInterval): Int =
        milliseconds.compareTo(other.milliseconds)

    override fun toString(): String =
        "$milliseconds ms"

    /** Returns the interval as a duration. */
    public fun asDuration(): Duration =
        milliseconds.milliseconds

    /** Returns the approximate frequency in hertz. */
    public fun toHz(): Double =
        1000.0 / milliseconds

    public companion object {

        public const val MIN_MS: Int = 10
        public const val MAX_MS: Int = 1000
        public val MIN: PulseInterval = PulseInterval(MIN_MS)
        public val MAX: PulseInterval = PulseInterval(MAX_MS)

        /** Creates a validated interval. */
        public fun of(milliseconds: Int): PulseInterval =
            when (milliseconds) {
                in MIN_MS..MAX_MS -> PulseInterval(milliseconds)
                else -> throw PatternException.InvalidInterval(milliseconds)
            }

        /** Converts an electrical frequency to an interval. */
        public fun fromHz(hz: Double): PulseInterval {
            if (!hz.isFinite() || hz <= 0.0) {
                throw PatternException.InvalidFrequency(hz)
            }

            val millis = round(1000.0 / hz)
            if (millis !in MIN_MS.toDouble()..MAX_MS.toDouble()) {
                throw PatternException.InvalidFrequency(hz)
            }
            return PulseInterval(millis.toInt())
        }

    }

}
