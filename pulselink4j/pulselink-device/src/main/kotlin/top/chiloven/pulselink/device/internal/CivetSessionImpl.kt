package top.chiloven.pulselink.device.internal

import top.chiloven.pulselink.core.DeviceKind
import top.chiloven.pulselink.core.DeviceState
import top.chiloven.pulselink.device.LocalEvent
import top.chiloven.pulselink.protocol.civet.Command50
import top.chiloven.pulselink.protocol.civet.Command66
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow

internal class CivetSessionImpl(private val base: BaseSession) {

    val kind: DeviceKind get() = DeviceKind.Civet
    val state: StateFlow<DeviceState> get() = base.state
    val events: SharedFlow<LocalEvent> get() = base.events

    suspend fun startPressure(color: Int = 0) {
        base.writeRaw(
            GattIds.CONTROL_WRITE,
            Command50(color, true).encode()
        )
    }

    suspend fun stopPressure(color: Int = 0) {
        base.writeRaw(
            GattIds.CONTROL_WRITE,
            Command50(color, false).encode()
        )
    }

    suspend fun resetPressure() {
        base.writeRaw(
            GattIds.CONTROL_WRITE,
            Command66(reset = true).encode(),
        )
    }

    suspend fun setOrientation(value: Int) {
        base.writeRaw(
            GattIds.CONTROL_WRITE,
            Command66(orientation = value).encode(),
        )
    }

    suspend fun shutdown(): Unit =
        base.shutdown()

    fun close(): Unit =
        base.close()

}
