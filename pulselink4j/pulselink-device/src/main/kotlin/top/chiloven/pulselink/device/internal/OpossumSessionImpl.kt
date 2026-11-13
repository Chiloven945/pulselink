package top.chiloven.pulselink.device.internal

import top.chiloven.pulselink.core.Channel
import top.chiloven.pulselink.core.DeviceKind
import top.chiloven.pulselink.core.DeviceState
import top.chiloven.pulselink.core.Intensity
import top.chiloven.pulselink.device.LocalEvent
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import top.chiloven.pulselink.protocol.opossum.B0 as OpossumB0
import top.chiloven.pulselink.protocol.opossum.B2 as OpossumB2
import top.chiloven.pulselink.protocol.opossum.B3 as OpossumB3

internal class OpossumSessionImpl(private val base: BaseSession) {

    val kind: DeviceKind get() = DeviceKind.Opossum
    val state: StateFlow<DeviceState> get() = base.state
    val events: SharedFlow<LocalEvent> get() = base.events

    suspend fun setIntensity(channel: Channel, intensity: Intensity) {
        val a = if (channel == Channel.A) intensity.value else null
        val b = if (channel == Channel.B) intensity.value else null
        base.writeRaw(
            GattIds.CONTROL_WRITE,
            OpossumB3(a, b).encode()
        )
        base.updateIntensity(channel, intensity)
    }

    suspend fun pulse(channel: Channel, values: List<Int>) {
        val a = if (channel == Channel.A) values else List(4) { 0 }
        val b = if (channel == Channel.B) values else List(4) { 0 }
        base.writeRaw(
            GattIds.CONTROL_WRITE,
            OpossumB0(a, b).encode()
        )
    }

    suspend fun display(a: Intensity, b: Intensity) {
        if ((base.link.mtu() ?: 0) >= 24) {
            base.writeRaw(
                GattIds.CONTROL_WRITE,
                OpossumB2(a.value, b.value).encode(),
            )
        }
    }

    suspend fun shutdown(): Unit =
        base.shutdown()

    fun close(): Unit =
        base.close()

}
