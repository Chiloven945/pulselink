package top.chiloven.pulselink.protocol.coyote.v3

import top.chiloven.pulselink.core.ChannelMap
import top.chiloven.pulselink.core.CoreException
import top.chiloven.pulselink.core.Intensity
import top.chiloven.pulselink.protocol.ProtocolException
import top.chiloven.pulselink.protocol.requireLength
import top.chiloven.pulselink.protocol.u8

/** A Coyote V3 B1 intensity response. */
public data class B1Message(
    public val sequence: Sequence,
    public val intensity: ChannelMap<Intensity>,
) {

    public companion object {

        /** Decodes a 4-byte B1 notification. */
        public fun decode(bytes: ByteArray): B1Message {
            bytes.requireLength(4)
            return when {
                bytes.u8(0) != 0xB1 -> throw ProtocolException.InvalidHeader(bytes.u8(0))
                else -> B1Message(
                    sequence = Sequence.of(bytes.u8(1)),
                    intensity = ChannelMap(
                        intensity("B1 channel A intensity", bytes.u8(2)),
                        intensity("B1 channel B intensity", bytes.u8(3)),
                    ),
                )
            }
        }

        private fun intensity(field: String, value: Int): Intensity =
            try {
                Intensity.of(value)
            } catch (_: CoreException.InvalidIntensity) {
                throw ProtocolException.ValueOutOfRange(field, value.toLong(), 0, 200)
            }

    }

}
