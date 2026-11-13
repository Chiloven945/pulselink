package top.chiloven.pulselink.ble.internal

import com.juul.kable.*
import top.chiloven.pulselink.transport.*
import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.TimeoutCancellationException
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.withTimeout

internal class KableConnector(
    private val advertisements: Map<TransportDeviceId, Advertisement>,
    private val peripherals: MutableMap<TransportDeviceId, Peripheral>,
    private val connectTimeout: Duration,
    private val operationTimeout: Duration,
) {

    suspend fun connect(device: DiscoveredDevice): GattLink {
        val peripheral = peripheralFor(device)
        try {
            return withTimeout(connectTimeout) {
                peripheral.connect()
                peripheral.state.first { it is State.Connected }
                val services = peripheral.services.first { it != null } ?: emptyList()
                val characteristics = buildMap {
                    for (service in services) {
                        for (characteristic in service.characteristics) {
                            val id = GattCharacteristic(
                                service.serviceUuid.toJavaUuid(),
                                characteristic.characteristicUuid.toJavaUuid(),
                            )
                            put(
                                id,
                                CharacteristicInfo(
                                    id = id,
                                    readable = characteristic.properties.read,
                                    writable = characteristic.properties.write,
                                    writableWithoutResponse = characteristic.properties.writeWithoutResponse,
                                    notifiable = characteristic.properties.notify,
                                    indicatable = characteristic.properties.indicate,
                                ),
                            )
                        }
                    }
                }
                val mtu = try {
                    peripheral.maximumWriteValueLengthForType(WriteType.WithResponse) + 3
                } catch (error: CancellationException) {
                    throw error
                } catch (_: Exception) {
                    null
                }
                KableGattLink(
                    device.id,
                    peripheral,
                    characteristics,
                    mtu,
                    operationTimeout
                )
            }
        } catch (_: TimeoutCancellationException) {
            throw TransportException.Timeout()
        } catch (error: CancellationException) {
            throw error
        } catch (error: TransportException) {
            throw error
        } catch (error: Exception) {
            throw TransportException.ConnectFailed(
                error.message ?: "connect failed",
                error
            )
        }
    }

    suspend fun reconnect(
        target: ReconnectTarget,
        scan: suspend (ScanFilter, Duration) -> List<DiscoveredDevice>,
    ): GattLink {
        val cached = peripherals[target.deviceId]
        if (cached != null) {
            return try {
                connect(
                    DiscoveredDevice(
                        target.deviceId,
                        null,
                        null,
                        emptyList()
                    )
                )
            } catch (_: TransportException) {
                reconnectByScan(target, scan)
            }
        }
        return reconnectByScan(target, scan)
    }

    fun close() {
        peripherals.values.forEach { peripheral -> runCatching { peripheral.close() } }
        peripherals.clear()
    }

    private suspend fun reconnectByScan(
        target: ReconnectTarget,
        scan: suspend (ScanFilter, Duration) -> List<DiscoveredDevice>,
    ): GattLink {
        val devices = scan(target.filter, 8.seconds)
        val device = devices.firstOrNull { it.id == target.deviceId } ?: devices.firstOrNull()
        ?: throw TransportException.DeviceNotFound(target.deviceId)
        return connect(device)
    }

    private fun peripheralFor(device: DiscoveredDevice): Peripheral =
        peripherals[device.id]
            ?: advertisements[device.id]
                    ?.let { advertisement ->
                        Peripheral(advertisement).also { peripherals[device.id] = it }
                    }
            ?: Peripheral(device.id.value.toIdentifier())
                    .also { peripherals[device.id] = it }

}
