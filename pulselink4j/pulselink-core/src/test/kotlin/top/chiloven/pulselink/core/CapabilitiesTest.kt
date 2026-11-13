package top.chiloven.pulselink.core

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse

class CapabilitiesTest {

    @Test
    fun capabilitiesDefaults() {
        assertEquals(ChannelSet.NONE, ChannelSet.NONE)
        assertEquals(2, ChannelSet.AB.count())
        assertFalse(DeviceCapabilities().battery)
    }

}
