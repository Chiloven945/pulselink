package top.chiloven.pulselink.ble

import com.juul.kable.*
import top.chiloven.pulselink.transport.*
import java.util.concurrent.ConcurrentHashMap
import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.first

public class KableBackend(
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO),
    private val connectTimeout: Duration = 15.seconds,
    private val operationTimeout: Duration = 5.seconds,
) : Backend, AutoCloseable {

    private val advertisements = ConcurrentHashMap<TransportDeviceId, Advertisement>()
    private val peripherals = ConcurrentHashMap<TransportDeviceId, Peripheral>()

    override suspend fun scan(
        filter: ScanFilter,
        timeout: Duration
    ): List<DiscoveredDevice> {
        val scanner = Scanner {
            if (filter.namePrefixes.isNotEmpty() || filter.serviceUuids.isNotEmpty()) {
                filters {
                    filter.serviceUuids.forEach { uuid ->
                        match { services = listOf(uuid.toKableUuid()) }
                    }
                    filter.namePrefixes.forEach { prefix ->
                        match { name = Filter.Name.Prefix(prefix) }
                    }
                }
            }
        }
        val found = linkedMapOf<TransportDeviceId, DiscoveredDevice>()
        try {
            withTimeoutOrNull(timeout) {
                scanner.advertisements.collect { advertisement ->
                    val device = advertisement.toDiscoveredDevice()
                    if (!filter.matches(device)) return@collect
                    advertisements[device.id] = advertisement
                    val previous = found[device.id]
                    found[device.id] = device.copy(
                        serviceUuids = (previous?.serviceUuids.orEmpty() + device.serviceUuids).distinct(),
                        manufacturerData = (previous?.manufacturerData.orEmpty() + device.manufacturerData)
                                .associateBy { it.companyId }.values.toList(),
                    )
                }
            }
        } catch (error: CancellationException) {
            throw error
        } catch (error: Exception) {
            throw TransportException.ScanFailed(error.message ?: "scan failed", error)
        }
        return found.values.toList()
    }

    override suspend fun connect(device: DiscoveredDevice): GattLink {
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
            throw TransportException.ConnectFailed(error.message ?: "connect failed", error)
        }
    }

    override suspend fun reconnect(target: ReconnectTarget): GattLink {
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
                reconnectByScan(target)
            }
        }
        return reconnectByScan(target)
    }

    private suspend fun reconnectByScan(target: ReconnectTarget): GattLink {
        val devices = scan(target.filter, 8.seconds)
        val device = devices.firstOrNull { it.id == target.deviceId } ?: devices.firstOrNull()
        ?: throw TransportException.DeviceNotFound(target.deviceId)
        return connect(device)
    }

    private fun peripheralFor(device: DiscoveredDevice): Peripheral =
        peripherals[device.id] ?: advertisements[device.id]?.let { advertisement ->
            Peripheral(advertisement).also { peripherals[device.id] = it }
        } ?: Peripheral(device.id.value.toIdentifier()).also { peripherals[device.id] = it }

    override fun close() {
        peripherals.values.forEach { peripheral -> runCatching { peripheral.close() } }
        peripherals.clear()
        advertisements.clear()
        scope.cancel()
    }

}
