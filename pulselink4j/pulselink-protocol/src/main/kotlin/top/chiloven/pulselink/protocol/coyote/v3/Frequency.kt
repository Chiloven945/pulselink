package top.chiloven.pulselink.protocol.coyote.v3

import top.chiloven.pulselink.pattern.PulseInterval
import top.chiloven.pulselink.protocol.ProtocolException

/** Converts a pulse interval to the Coyote V3 wire value. */
public fun encodeInterval(interval: PulseInterval): Int =
    when (val millis = interval.milliseconds) {
        in 10..100 -> millis
        in 101..600 -> (millis - 100) / 5 + 100
        in 601..1000 -> (millis - 600) / 10 + 200
        else -> 10
    }

/** Converts a Coyote V3 wire value to a representative pulse interval. */
public fun decodeWireFrequency(value: Int): PulseInterval =
    when (value) {
        !in 10..240 -> {
            throw ProtocolException.ValueOutOfRange(
                "wire frequency",
                value.toLong(),
                10,
                240
            )
        }

        else -> PulseInterval.of(
            when (value) {
                in 10..100 -> value
                in 101..200 -> 100 + (value - 100) * 5
                else -> 600 + (value - 200) * 10
            }
        )
    }
