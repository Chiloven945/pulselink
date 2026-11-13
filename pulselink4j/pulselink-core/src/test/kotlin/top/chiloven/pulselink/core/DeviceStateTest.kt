package top.chiloven.pulselink.core

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class DeviceStateTest {

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
    fun observedDistinguishesUnknownFromZero() {
        val unknown: Observed<Intensity> = Observed.Unknown
        val zero: Observed<Intensity> = Observed.Known(Intensity.MIN)
        assertEquals(null, unknown.valueOrNull())
        assertEquals(Intensity.MIN, zero.valueOrNull())
        assertEquals(Observed.Unknown, unknown)
        assertEquals(Observed.Known(Intensity.MIN), zero)
    }

}
