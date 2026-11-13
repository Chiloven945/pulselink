package top.chiloven.pulselink.device.internal

import top.chiloven.pulselink.core.BatteryLevel
import top.chiloven.pulselink.protocol.ProtocolException
import top.chiloven.pulselink.protocol.coyote.v3.decodeBattery
import top.chiloven.pulselink.transport.GattCharacteristic
import top.chiloven.pulselink.transport.TransportException
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch

internal class SessionBattery(private val base: BaseSession) {

    fun subscribe(scope: CoroutineScope): Job =
        scope.launch {
            base.link.subscribe(
                GattCharacteristic(
                    GattIds.BATTERY_SERVICE,
                    GattIds.BATTERY_CHAR
                )
            ).collect { bytes ->
                decode(bytes)?.let(base::updateBattery)
            }
        }

    suspend fun readLevel(): BatteryLevel? {
        val bytes = try {
            base.link.read(
                GattCharacteristic(
                    GattIds.BATTERY_SERVICE,
                    GattIds.BATTERY_CHAR
                )
            )
        } catch (error: CancellationException) {
            throw error
        } catch (_: TransportException) {
            return null
        }
        return decode(bytes)
    }

    private fun decode(bytes: ByteArray): BatteryLevel? =
        try {
            decodeBattery(bytes)
        } catch (_: ProtocolException) {
            null
        }

}
