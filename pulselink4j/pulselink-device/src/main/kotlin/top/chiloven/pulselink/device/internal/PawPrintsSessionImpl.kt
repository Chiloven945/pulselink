package top.chiloven.pulselink.device.internal

import top.chiloven.pulselink.core.DeviceKind
import top.chiloven.pulselink.core.DeviceState
import top.chiloven.pulselink.device.LocalEvent
import top.chiloven.pulselink.protocol.pawprints.*
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow

internal class PawPrintsSessionImpl(private val base: BaseSession) {

    val kind: DeviceKind get() = DeviceKind.PawPrints
    val state: StateFlow<DeviceState> get() = base.state
    val events: SharedFlow<LocalEvent> get() = base.events

    suspend fun noTrigger(color: Int = 0) {
        base.writeRaw(
            GattIds.CONTROL_WRITE,
            Command50.noTrigger(color).asBytes()
        )
    }

    suspend fun configure(command: Command50) {
        base.writeRaw(GattIds.CONTROL_WRITE, command.asBytes())
    }

    suspend fun resetParameter() {
        base.writeRaw(
            GattIds.CONTROL_WRITE,
            COMMAND_RESET_PARAMETER,
        )
    }

    suspend fun autoDetectAngles() {
        base.writeRaw(
            GattIds.CONTROL_WRITE,
            COMMAND_AUTO_DETECT_ANGLES,
        )
    }

    suspend fun shoulderLight(color: Int) {
        base.writeRaw(
            GattIds.CONTROL_WRITE,
            commandShoulderLight(color)
        )
    }

    suspend fun shoulderLightFlash(
        colorA: Int,
        colorB: Int,
        speed: Int
    ) {
        base.writeRaw(
            GattIds.CONTROL_WRITE,
            commandShoulderLightFlash(colorA, colorB, speed),
        )
    }

    suspend fun shutdown(): Unit =
        base.shutdown()

    fun close(): Unit =
        base.close()

}
