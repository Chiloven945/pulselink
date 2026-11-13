package top.chiloven.pulselink.protocol.pawprints

import top.chiloven.pulselink.protocol.ProtocolException
import top.chiloven.pulselink.protocol.u8

public sealed interface Message {

    public data class Info(public val message: Message51) : Message

    public data class Activated(public val message: Message5a) : Message

    public data class Deactivated(public val message: Message5b) : Message

    public data class Parameter(public val message: Message5c) : Message

    public data class Sensor(public val message: D0Message) : Message

    public data class Angles(public val message: F1Message) : Message

    public companion object {

        public fun decode(bytes: ByteArray): Message =
            when (bytes.firstOrNull()?.toInt()?.and(0xFF)) {
                0x51 if bytes.size == 4 -> Info(
                    Message51(
                        bytes.u8(1),
                        bytes.u8(2),
                        bytes.u8(3)
                    )
                )

                0x5A if bytes.size == 4 -> Activated(
                    Message5a(
                        bytes.u8(1),
                        bytes.u8(2),
                        bytes.u8(3)
                    )
                )

                0x5B if bytes.size == 3 -> Deactivated(
                    Message5b(
                        bytes.u8(1),
                        bytes.u8(2)
                    )
                )

                0x5C if bytes.size == 4 -> Parameter(
                    Message5c(
                        bytes.u8(1),
                        bytes.u8(2),
                        bytes.u8(3)
                    )
                )

                0xD0 if bytes.size == 9 -> Sensor(
                    D0Message(
                        color = bytes.u8(1),
                        sequence = bytes.u8(2),
                        button = bytes.u8(3),
                        acceleration = bytes.u8(4),
                        x = bytes.u8(5).toByte().toInt(),
                        y = bytes.u8(6).toByte().toInt(),
                        z = bytes.u8(7).toByte().toInt(),
                        voltage = bytes.u8(8),
                    ),
                )

                0xF1 if bytes.size == 14 && bytes.u8(1) == 0x61 -> Angles(
                    F1Message(
                        pair(bytes, 2),
                        pair(bytes, 6),
                        pair(bytes, 10)
                    ),
                )

                null -> throw ProtocolException.InvalidLength(4, 0)

                else -> throw ProtocolException.InvalidHeader(bytes.u8(0))
            }

        private fun pair(
            bytes: ByteArray,
            offset: Int
        ): Pair<Int, Int> {
            val lower = (bytes.u8(offset) or (bytes.u8(offset + 1) shl 8))
                    .toShort()
                    .toInt() / 2
            val upper = (bytes.u8(offset + 2) or (bytes.u8(offset + 3) shl 8))
                    .toShort()
                    .toInt() / 2
            return lower.coerceIn(-128, 127) to upper.coerceIn(-128, 127)
        }

    }

}
