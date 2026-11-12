package top.chiloven.pulselink.protocol

import top.chiloven.pulselink.pattern.PulseInterval
import top.chiloven.pulselink.protocol.coyote.v2.*
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class V2Test {

    @Test
    fun officialPulseVectors() {
        assertEquals(
            listOf(0x21, 0x01, 0x00),
            PwmPulse(1, 9, 0).encode().map { it.toInt() and 0xFF })
        assertEquals(
            listOf(0x41, 0x81, 0x01),
            PwmPulse(1, 10, 3).encode().map { it.toInt() and 0xFF })
        assertEquals(
            PwmPulse(1, 10, 3),
            PwmPulse.decode(byteArrayOf(0x41, 0x81.toByte(), 0x01))
        )
    }

    @Test
    fun intensityBitLayout() {
        val value = PwmAb2(
            CoyoteV2Intensity.of(10),
            CoyoteV2Intensity.of(20)
        )
        assertEquals(
            listOf(0x14, 0x50, 0x00),
            value.encode().map { it.toInt() and 0xFF }
        )
        assertEquals(
            value,
            PwmAb2.decode(value.encode())
        )
    }

    @Test
    fun boundsAreValidated() {
        assertFailsWith<ProtocolException.ValueOutOfRange> {
            PwmPulse(32, 0, 0)
        }
        assertFailsWith<ProtocolException.ValueOutOfRange> {
            PwmPulse(0, 1024, 0)
        }
        assertFailsWith<ProtocolException.ValueOutOfRange> {
            PwmPulse(0, 0, 32)
        }
        assertFailsWith<ProtocolException.ValueOutOfRange> {
            CoyoteV2Intensity.of(2048)
        }
        assertEquals(
            1400,
            CoyoteV2Intensity.fromDisplay(200).value
        )
    }

    @Test
    fun intervalConversion() {
        val pulse = encodeInterval(PulseInterval.of(10), 0)
        assertEquals(1, pulse.x)
        assertEquals(9, pulse.y)
        assertEquals(
            10,
            decodeInterval(pulse).milliseconds
        )
        assertEquals(
            31,
            encodeInterval(PulseInterval.of(100), 40).z
        )
    }

}
