package top.chiloven.pulselink.core

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class BatteryTest {

    @Test
    fun batteryBounds() {
        assertEquals(0, BatteryLevel.MIN.value)
        assertEquals(100, BatteryLevel.MAX.value)
        assertFailsWith<CoreException.InvalidBatteryLevel> { BatteryLevel.of(101) }
    }

}
