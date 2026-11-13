package top.chiloven.pulselink.protocol

import top.chiloven.pulselink.core.ButtonMask
import top.chiloven.pulselink.protocol.opossum.*
import kotlin.test.Test
import kotlin.test.assertEquals

class OpossumTest {

    @Test
    fun opossumLayouts() {
        val b0 = B0(
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
            (Message.decode(
                byteArrayOf(
                    0xB3.toByte(),
                    10,
                    20
                )
            ) as Message.B3Value).message
        )
        val d0 = Message.decode(
            byteArrayOf(
                0xD0.toByte(),
                7, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            )
        )
        assertEquals(
            ButtonMask.fromBits(2),
            (d0 as Message.D0Value).message.buttons
        )
    }

}
