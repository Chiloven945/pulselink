package top.chiloven.pulselink.core

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class PressureTest {

    @Test
    fun pressureConversion() {
        assertEquals(790, Pressure.fromKpa(7.9f)?.centiKpa)
        assertEquals(7.9f, Pressure.fromKpa(7.9f)?.asKpa())
        assertNull(Pressure.fromKpa(Float.NaN))
        assertNull(Pressure.fromKpa(Float.POSITIVE_INFINITY))
    }

}
