package top.chiloven.pulselink.core

import kotlin.test.Test
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class ButtonTest {

    @Test
    fun buttonMask() {
        val mask = ButtonMask.fromBits(0b101)
        assertTrue(mask.contains(ButtonMask.fromBits(0b001)))
        assertFalse(mask.isEmpty())
        assertTrue(ButtonMask.NONE.isEmpty())
        assertFailsWith<CoreException.InvalidButtonMask> { ButtonMask.fromBits(0x1_0000) }
    }

}
