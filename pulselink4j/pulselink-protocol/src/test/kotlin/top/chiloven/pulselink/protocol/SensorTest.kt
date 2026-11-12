package top.chiloven.pulselink.protocol

import top.chiloven.pulselink.core.ButtonMask
import top.chiloven.pulselink.core.Pressure
import top.chiloven.pulselink.protocol.civet.Command66
import top.chiloven.pulselink.protocol.opossum.B2
import top.chiloven.pulselink.protocol.opossum.B3
import top.chiloven.pulselink.protocol.opossum.B3Message
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import top.chiloven.pulselink.protocol.civet.Command50 as CivetCommand50
import top.chiloven.pulselink.protocol.civet.D0Message as CivetD0
import top.chiloven.pulselink.protocol.opossum.B0 as OpossumB0
import top.chiloven.pulselink.protocol.opossum.Message as OpossumMessage
import top.chiloven.pulselink.protocol.pawprints.Command50 as PawCommand50
import top.chiloven.pulselink.protocol.pawprints.Message as PawMessage

class SensorTest {

    @Test
    fun civetLayouts() {
        val start = CivetCommand50(1, true).encode()
        assertEquals(17, start.size)
        assertEquals(
            listOf(0x50, 1, 0xD0),
            start.take(3).map { it.toInt() and 0xFF }
        )
        val reset = Command66(reset = true).encode()
        assertEquals(13, reset.size)
        assertEquals(2, reset[12].toInt() and 0xFF)
        val bytes = ByteArray(17)
        bytes[0] = 0xD0.toByte()
        bytes[8] = 0x16
        bytes[9] = 0x03
        assertEquals(
            Pressure.fromCentiKpa(790),
            CivetD0.decode(bytes).pressure
        )
    }

    @Test
    fun opossumLayouts() {
        val b0 = OpossumB0(
            listOf(0, 25, 50, 100),
            listOf(100, 50, 25, 0)
        ).encode()
        assertEquals(20, b0.size)
        assertEquals(
            listOf(0, 25, 50, 100),
            b0.slice(8..11).map { it.toInt() and 0xFF }
        )
        assertEquals(
            listOf(0xB3, 160, 0xFF),
            B3(160, null).encode().map { it.toInt() and 0xFF }
        )
        val b2 = B2(10, 20).encode()
        assertEquals(24, b2.size)
        assertEquals(
            listOf(10, 20),
            b2.takeLast(2).map { it.toInt() and 0xFF }
        )
        assertEquals(
            B3Message(10, 20),
            (OpossumMessage.decode(
                byteArrayOf(
                    0xB3.toByte(),
                    10,
                    20
                )
            ) as OpossumMessage.B3Value).message
        )
        val d0 = OpossumMessage.decode(
            byteArrayOf(
                0xD0.toByte(),
                7, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            )
        )
        assertEquals(
            ButtonMask.fromBits(2),
            (d0 as OpossumMessage.D0Value).message.buttons
        )
    }

    @Test
    fun pawPrintsLayouts() {
        assertEquals(
            listOf(0x50, 1, 0x00),
            PawCommand50.noTrigger(1).asBytes()
                    .take(3)
                    .map { it.toInt() and 0xFF })
        val probability = PawCommand50.probability(
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
        val activated = PawMessage.decode(byteArrayOf(0x5A, 1, 7, 42))
        assertEquals(
            7,
            (activated as PawMessage.Activated).message.eventId
        )
        val sensor = PawMessage.decode(
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
            (sensor as PawMessage.Sensor).message.x
        )
        assertFailsWith<ProtocolException.InvalidHeader> {
            PawMessage.decode(byteArrayOf(0x01))
        }
    }

}
