package top.chiloven.pulselink.protocol.coyote.v2

import top.chiloven.pulselink.pattern.PulseInterval
import top.chiloven.pulselink.protocol.ProtocolException
import kotlin.math.floor
import kotlin.math.sqrt

/** Default pulse width parameter; `20` corresponds to 100 microseconds. */
public const val DEFAULT_PULSE_WIDTH: Int = 20

/** Converts a pulse interval to recommended V2 `[X, Y, Z]` values. */
public fun encodeInterval(
    interval: PulseInterval,
    width: Int = DEFAULT_PULSE_WIDTH
): PwmPulse {
    val frequency = interval.milliseconds
    val suggested = sqrt(frequency / 1000.0) * 15.0
    val x = floor(suggested).toInt().coerceIn(1, 31)
    val y = (frequency - x)
            .coerceAtLeast(0)
            .coerceAtMost(1023)

    return PwmPulse(x, y, width.coerceIn(0, 31))
}

/** Decodes a V2 pulse into its representative interval. */
public fun decodeInterval(pulse: PwmPulse): PulseInterval {
    val interval = pulse.x + pulse.y
    return try {
        PulseInterval.of(interval)
    } catch (_: Exception) {
        throw ProtocolException.ValueOutOfRange(
            "Coyote V2 pulse interval",
            interval.toLong(),
            10,
            1000
        )
    }
}
