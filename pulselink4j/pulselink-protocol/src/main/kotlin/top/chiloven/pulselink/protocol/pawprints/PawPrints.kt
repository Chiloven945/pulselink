package top.chiloven.pulselink.protocol.pawprints

import top.chiloven.pulselink.protocol.ProtocolException
import top.chiloven.pulselink.protocol.toU8
import top.chiloven.pulselink.protocol.u8
import java.util.*

public val SERVICE_CONTROL: UUID = UUID.fromString("0000180c-0000-1000-8000-00805f9b34fb")
public val CHAR_WRITE: UUID = UUID.fromString("0000150a-0000-1000-8000-00805f9b34fb")
public val CHAR_NOTIFY: UUID = UUID.fromString("0000150b-0000-1000-8000-00805f9b34fb")
public const val DEVICE_NAME_V1_1: String = "47L120300"
public const val DEVICE_NAME_V1_0: String = "47L120100"

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

public val COMMAND_RESET_PARAMETER: ByteArray = byteArrayOf(0x5F.toU8())
public val COMMAND_AUTO_DETECT_ANGLES: ByteArray = byteArrayOf(0x60.toU8())

public fun commandShoulderLight(color: Int): ByteArray =
    byteArrayOf(
        0x70.toU8(),
        color.toU8()
    )

public fun commandShoulderLightFlash(
    colorA: Int,
    colorB: Int,
    speed: Int
): ByteArray =
    byteArrayOf(
        0x70.toU8(),
        colorA.toU8(),
        colorB.toU8(),
        speed.toU8()
    )

public data class Message51(
    public val color: Int,
    public val deviceType: Int,
    public val battery: Int
)

public data class Message5a(
    public val color: Int,
    public val eventId: Int,
    public val parameter: Int
)

public data class Message5b(
    public val color: Int,
    public val eventId: Int
)

public data class Message5c(
    public val color: Int,
    public val eventId: Int,
    public val parameter: Int
)

public data class D0Message(
    public val color: Int,
    public val sequence: Int,
    public val button: Int,
    public val acceleration: Int,
    public val x: Int,
    public val y: Int,
    public val z: Int,
    public val voltage: Int,
)

public data class F1Message(
    public val x: Pair<Int, Int>,
    public val y: Pair<Int, Int>,
    public val z: Pair<Int, Int>,
)

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
