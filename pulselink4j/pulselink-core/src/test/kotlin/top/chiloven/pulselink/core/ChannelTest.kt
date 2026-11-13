package top.chiloven.pulselink.core

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class ChannelTest {

    @Test
    fun channelDecode() {
        assertEquals(Channel.A, Channel.fromWire(0))
        assertEquals(Channel.B, Channel.fromWire(1))
        assertFailsWith<CoreException.InvalidChannel> { Channel.fromWire(2) }
    }

}
