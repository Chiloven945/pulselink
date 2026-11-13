package top.chiloven.pulselink.protocol.pawprints

import top.chiloven.pulselink.protocol.toU8

/** A 17-byte `0x50` trigger configuration command. */
public class Command50 private constructor(private val bytes: ByteArray) {

    /** Returns a copy of the command bytes. */
    public fun asBytes(): ByteArray =
        bytes.copyOf()

    public companion object {

        private const val COMMAND_LEN: Int = 17

        /** No-trigger configuration, required immediately after connecting. */
        public fun noTrigger(color: Int): Command50 =
            Command50(ByteArray(COMMAND_LEN).also {
                it[0] = 0x50.toU8()
                it[1] = color.toU8()
            })

        public fun randomReaction(
            color: Int,
            eventId: Int,
            delayRange: IntRange,
            reactionSeconds: Int,
            increaseRate: Int,
            decreaseOnDeactivation: Int,
            decreaseRate: Int,
            increaseOnActivation: Int,
        ): Command50 {
            require(delayRange.first in 0..65535 && delayRange.last in 0..65535) {
                "delay range must fit u16"
            }
            return Command50(ByteArray(COMMAND_LEN).also {
                it[0] = 0x50.toU8()
                it[1] = color.toU8()
                it[2] = 0x03.toU8()
                it[3] = eventId.toU8()
                putLe16(it, 4, delayRange.first)
                putLe16(it, 6, delayRange.last)
                putLe16(it, 8, reactionSeconds)
                it[10] = increaseRate.toU8()
                it[11] = decreaseOnDeactivation.toU8()
                it[12] = decreaseRate.toU8()
                it[13] = increaseOnActivation.toU8()
            })
        }

        public fun probability(
            color: Int,
            events: List<Pair<Int, Int>>,
            cooldownSeconds: Int
        ): Command50 {
            require(events.size <= 6) { "at most 6 probability events" }
            return Command50(ByteArray(COMMAND_LEN).also {
                it[0] = 0x50.toU8()
                it[1] = color.toU8()
                it[2] = 0x04.toU8()
                events.forEachIndexed { index, (event, probability) ->
                    it[3 + index * 2] = event.toU8()
                    it[4 + index * 2] = probability.toU8()
                }
                putLe16(it, 15, cooldownSeconds)
            })
        }

        public fun externalVoltage(
            color: Int,
            eventId: Int,
            highImpedance: Boolean,
            targetRange: Pair<Int, Int>,
            mappingRange: Int,
        ): Command50 = Command50(ByteArray(COMMAND_LEN).also {
            it[0] = 0x50.toU8()
            it[1] = color.toU8()
            it[2] = 0x0F.toU8()
            it[3] = eventId.toU8()
            it[4] = (if (highImpedance) 1 else 0).toU8()
            it[5] = targetRange.first.toU8()
            it[6] = targetRange.second.toU8()
            it[7] = mappingRange.toU8()
        })

        public fun rawSensor(color: Int): Command50 =
            Command50(ByteArray(COMMAND_LEN).also {
                it[0] = 0x50.toU8()
                it[1] = color.toU8()
                it[2] = 0xD0.toU8()
                it[3] = 0x01.toU8()
            })

        private fun putLe16(
            bytes: ByteArray,
            offset: Int,
            value: Int
        ) {
            bytes[offset] = (value and 0xFF).toByte()
            bytes[offset + 1] = ((value ushr 8) and 0xFF).toByte()
        }

    }

}
