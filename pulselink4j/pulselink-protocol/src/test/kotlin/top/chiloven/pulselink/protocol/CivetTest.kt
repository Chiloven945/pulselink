package top.chiloven.pulselink.protocol

import top.chiloven.pulselink.core.Pressure
import top.chiloven.pulselink.protocol.civet.Command50
import top.chiloven.pulselink.protocol.civet.Command66
import top.chiloven.pulselink.protocol.civet.D0Message
import kotlin.test.Test
import kotlin.test.assertEquals

class CivetTest {

    @Test
    fun civetLayouts() {
        val start = Command50(1, true).encode()
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
            D0Message.decode(bytes).pressure
        )
    }

}
