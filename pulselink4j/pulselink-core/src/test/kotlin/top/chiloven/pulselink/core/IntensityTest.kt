package top.chiloven.pulselink.core

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull

class IntensityTest {

    @Test
    fun intensityBounds() {
        assertEquals(0, Intensity.MIN.value)
        assertEquals(200, Intensity.MAX.value)
        assertEquals(10, Intensity.of(10).value)
        assertFailsWith<CoreException.InvalidIntensity> { Intensity.of(-1) }
        assertFailsWith<CoreException.InvalidIntensity> { Intensity.of(201) }
        assertEquals(
            200,
            Intensity.of(190).saturatingAdd(20).value
        )
        assertNull(Intensity.of(190).checkedAdd(20))
    }

    @Test
    fun intensityDeltaBounds() {
        assertEquals(-200, IntensityDelta.MIN.value)
        assertEquals(200, IntensityDelta.MAX.value)
        assertFailsWith<CoreException.InvalidIntensityDelta> { IntensityDelta.of(-201) }
        assertEquals(7, IntensityDelta.of(-7).magnitude())
    }

}
