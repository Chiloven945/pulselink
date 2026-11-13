package top.chiloven.pulselink.runtime.internal

import top.chiloven.pulselink.core.ChannelMap
import top.chiloven.pulselink.core.DeviceEvent
import top.chiloven.pulselink.core.Observed
import top.chiloven.pulselink.protocol.coyote.v3.B1Message
import top.chiloven.pulselink.runtime.CoyoteV3Event

internal class NotificationRouter(private val core: SessionCore) {

    suspend fun handle(bytes: ByteArray) {
        if (bytes.isEmpty() || bytes[0].toInt() and 0xFF != 0xB1) return
        val message = runCatching { B1Message.decode(bytes) }.getOrElse { error ->
            core.emit(CoyoteV3Event.Device(DeviceEvent.Error(error.message ?: "invalid B1")))
            return
        }
        val known = core.currentState().common.channels.map {
            it.copy(intensity = Observed.Unknown)
        }
        val a = known.a.copy(intensity = Observed.Known(message.intensity.a))
        val b = known.b.copy(intensity = Observed.Known(message.intensity.b))
        core.updateState {
            it.copy(
                common = it.common.copy(
                    channels = ChannelMap(a, b),
                    revision = it.common.revision + 1,
                ),
                lastB1 = message,
            )
        }
        if (message.sequence.value != 0) {
            core.ackTracker.complete(message.sequence)
        }
    }

}
