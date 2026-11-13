package top.chiloven.pulselink.runtime.internal

import top.chiloven.pulselink.core.ConnectionState
import top.chiloven.pulselink.core.DeviceEvent
import top.chiloven.pulselink.core.Observed
import top.chiloven.pulselink.protocol.coyote.v3.*
import top.chiloven.pulselink.runtime.CoyoteV3Balance
import top.chiloven.pulselink.runtime.CoyoteV3Event
import top.chiloven.pulselink.runtime.CoyoteV3Settings
import top.chiloven.pulselink.transport.GattCharacteristic
import top.chiloven.pulselink.transport.WriteMode
import kotlinx.coroutines.launch

internal class Initialization(private val core: SessionCore) {

    suspend fun run() {
        core.updateState {
            it.copy(common = it.common.copy(connection = ConnectionState.Connecting))
        }
        core.notifyJob?.cancel()
        core.batteryJob?.cancel()
        try {
            core.batteryJob = core.scope.launch {
                core.link.subscribe(
                    GattCharacteristic(SERVICE_BATTERY, CHAR_BATTERY)
                ).collect { bytes ->
                    runCatching { decodeBattery(bytes) }.onSuccess { battery ->
                        core.updateBattery(battery)
                    }
                }
            }
            core.notifyJob = core.scope.launch {
                core.link.subscribe(
                    GattCharacteristic(SERVICE_CONTROL, CHAR_NOTIFY)
                ).collect { bytes ->
                    core.notificationRouter.handle(bytes)
                }
            }
            val battery = core.link.read(
                GattCharacteristic(SERVICE_BATTERY, CHAR_BATTERY)
            )
            core.updateBattery(decodeBattery(battery))
            val balance = core.config.balance ?: CoyoteV3Balance()
            val bf = BfCommand(
                core.config.initialSoftLimit,
                balance.frequency,
                balance.intensity
            )
            core.link.write(
                GattCharacteristic(SERVICE_CONTROL, CHAR_WRITE),
                bf.encode(),
                WriteMode.Auto,
            )
            core.updateState {
                it.copy(
                    common = it.common.copy(connection = ConnectionState.Ready).touch(),
                    settings = Observed.Known(
                        CoyoteV3Settings(
                            core.config.initialSoftLimit,
                            balance.frequency,
                            balance.intensity,
                        ),
                    ),
                )
            }
        } catch (error: Throwable) {
            core.updateState {
                it.copy(common = it.common.copy(connection = ConnectionState.Degraded).touch())
            }
            core.emit(
                CoyoteV3Event.Device(
                    DeviceEvent.Error(
                        error.message ?: "initialization failed"
                    )
                )
            )
            throw error
        }
    }

}
