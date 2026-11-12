package top.chiloven.pulselink.protocol.coyote.v3

import top.chiloven.pulselink.core.BatteryLevel
import top.chiloven.pulselink.core.CoreException
import top.chiloven.pulselink.protocol.ProtocolException
import top.chiloven.pulselink.protocol.requireLength
import top.chiloven.pulselink.protocol.u8

/** Decodes a 1-byte battery notification. */
public fun decodeBattery(bytes: ByteArray): BatteryLevel {
    bytes.requireLength(1)
    val value = bytes.u8(0)
    return try {
        BatteryLevel.of(value)
    } catch (_: CoreException.InvalidBatteryLevel) {
        throw ProtocolException.ValueOutOfRange(
            "battery level",
            value.toLong(),
            0,
            100
        )
    }
}
