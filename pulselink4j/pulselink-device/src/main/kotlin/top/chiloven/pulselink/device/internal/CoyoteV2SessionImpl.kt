package top.chiloven.pulselink.device.internal

import top.chiloven.pulselink.core.Channel
import top.chiloven.pulselink.core.DeviceKind
import top.chiloven.pulselink.core.DeviceState
import top.chiloven.pulselink.core.Intensity
import top.chiloven.pulselink.device.LocalEvent
import top.chiloven.pulselink.pattern.PulseInterval
import top.chiloven.pulselink.protocol.coyote.v2.CoyoteV2Intensity
import top.chiloven.pulselink.protocol.coyote.v2.PwmAb2
import top.chiloven.pulselink.protocol.coyote.v2.PwmPulse
import top.chiloven.pulselink.protocol.coyote.v2.encodeInterval
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow

internal class CoyoteV2SessionImpl(private val base: BaseSession) {

    val kind: DeviceKind get() = DeviceKind.CoyoteV2
    val state: StateFlow<DeviceState> get() = base.state
    val events: SharedFlow<LocalEvent> get() = base.events

    suspend fun setIntensity(channel: Channel, intensity: Intensity) {
        val value = CoyoteV2Intensity.fromDisplay(intensity.value)
        val current = currentWire()
        val next = when (channel) {
            Channel.A -> PwmAb2(value, current.b)
            Channel.B -> PwmAb2(current.a, value)
        }
        base.writeRaw(GattIds.PWM_AB2, next.encode())
        base.updateIntensity(channel, intensity)
    }

    suspend fun setPulse(channel: Channel, pulse: PwmPulse) {
        val characteristic = when (channel) {
            Channel.A -> GattIds.PWM_A34
            Channel.B -> GattIds.PWM_B34
        }
        base.writeRaw(characteristic, pulse.encode())
    }

    suspend fun setPulse(
        channel: Channel,
        interval: PulseInterval
    ) {
        setPulse(channel, encodeInterval(interval))
    }

    suspend fun shutdown(): Unit =
        base.shutdown()

    fun close(): Unit =
        base.close()

    private fun currentWire(): PwmAb2 {
        val a = state.value.channel(Channel.A).intensity.valueOrNull()?.value ?: 0
        val b = state.value.channel(Channel.B).intensity.valueOrNull()?.value ?: 0
        return PwmAb2(
            CoyoteV2Intensity.fromDisplay(a),
            CoyoteV2Intensity.fromDisplay(b)
        )
    }

}
