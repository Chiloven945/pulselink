package top.chiloven.pulselink.core

import kotlin.test.*

class CoreTest {

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

    @Test
    fun batteryBounds() {
        assertEquals(0, BatteryLevel.MIN.value)
        assertEquals(100, BatteryLevel.MAX.value)
        assertFailsWith<CoreException.InvalidBatteryLevel> { BatteryLevel.of(101) }
    }

    @Test
    fun channelDecode() {
        assertEquals(Channel.A, Channel.fromWire(0))
        assertEquals(Channel.B, Channel.fromWire(1))
        assertFailsWith<CoreException.InvalidChannel> { Channel.fromWire(2) }
    }

    @Test
    fun pressureConversion() {
        assertEquals(790, Pressure.fromKpa(7.9f)?.centiKpa)
        assertEquals(7.9f, Pressure.fromKpa(7.9f)?.asKpa())
        assertNull(Pressure.fromKpa(Float.NaN))
        assertNull(Pressure.fromKpa(Float.POSITIVE_INFINITY))
    }

    @Test
    fun buttonMask() {
        val mask = ButtonMask.fromBits(0b101)
        assertTrue(mask.contains(ButtonMask.fromBits(0b001)))
        assertFalse(mask.isEmpty())
        assertTrue(ButtonMask.NONE.isEmpty())
        assertFailsWith<CoreException.InvalidButtonMask> { ButtonMask.fromBits(0x1_0000) }
    }

    @Test
    fun observedDistinguishesUnknownFromZero() {
        val unknown: Observed<Intensity> = Observed.Unknown
        val zero: Observed<Intensity> = Observed.Known(Intensity.MIN)
        assertEquals(null, unknown.valueOrNull())
        assertEquals(Intensity.MIN, zero.valueOrNull())
        assertEquals(Observed.Unknown, unknown)
        assertEquals(Observed.Known(Intensity.MIN), zero)
    }

    @Test
    fun deviceKindRemoteMapping() {
        assertEquals(
            DeviceKind.CoyoteV2,
            DeviceKind.fromRemoteType("COYOTE_020")
        )
        assertEquals(
            DeviceKind.CoyoteV3,
            DeviceKind.fromRemoteType("COYOTE_030")
        )
        assertEquals(
            DeviceKind.Civet,
            DeviceKind.fromRemoteType("BMTR_1")
        )
        assertEquals(
            DeviceKind.Opossum,
            DeviceKind.fromRemoteType("OVC_1")
        )
        assertEquals(
            DeviceKind.Unknown("X"),
            DeviceKind.fromRemoteType("X")
        )
        assertEquals(
            "COYOTE_030",
            DeviceKind.CoyoteV3.remoteType
        )
        assertNull(DeviceKind.PawPrints.remoteType)
    }

    @Test
    fun deviceStateRevisionAndChannels() {
        val state = DeviceState.empty(ConnectionState.Connecting)
        assertEquals(0, state.revision)
        assertEquals(1, state.touch().revision)
        assertEquals(2, state.touch().touch().revision)
        assertTrue(state.channel(Channel.A).intensity is Observed.Unknown)
        val updated = state.copy(
            channels = state.channels.map {
                ChannelState(intensity = Observed.Known(Intensity.of(10)))
            },
        )
        assertEquals(
            Intensity.of(10),
            updated.channel(Channel.A).intensity.valueOrNull()
        )
    }

    @Test
    fun capabilitiesDefaults() {
        assertEquals(ChannelSet.NONE, ChannelSet.NONE)
        assertEquals(2, ChannelSet.AB.count())
        assertFalse(DeviceCapabilities().battery)
    }

}
