package top.chiloven.pulselink.protocol

import top.chiloven.pulselink.protocol.pawprints.Command50
import top.chiloven.pulselink.protocol.pawprints.Message
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class PawPrintsTest {

    @Test
    fun pawPrintsLayouts() {
        assertEquals(
            listOf(0x50, 1, 0x00),
            Command50.noTrigger(1).asBytes()
                    .take(3)
                    .map { it.toInt() and 0xFF })
        val probability = Command50.probability(
            1,
            listOf(5 to 60, 2 to 80),
            600
        ).asBytes()
        assertEquals(
            listOf(0x04, 5, 60, 2),
            probability
                    .slice(2..5)
                    .map { it.toInt() and 0xFF }
        )
        assertEquals(
            600,
            (probability[15].toInt() and 0xFF) or
                    ((probability[16].toInt() and 0xFF) shl 8)
        )
        val activated = Message.decode(byteArrayOf(0x5A, 1, 7, 42))
        assertEquals(
            7,
            (activated as Message.Activated).message.eventId
        )
        val sensor = Message.decode(
            byteArrayOf(
                0xD0.toByte(),
                1,
                7,
                1,
                10,
                0xFF.toByte(),
                2,
                3,
                9
            )
        )
        assertEquals(
            -1,
            (sensor as Message.Sensor).message.x
        )
        assertFailsWith<ProtocolException.InvalidHeader> {
            Message.decode(byteArrayOf(0x01))
        }
    }

}
