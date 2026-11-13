package top.chiloven.pulselink.ble.internal

import com.juul.kable.Advertisement
import com.juul.kable.Filter
import com.juul.kable.Scanner
import top.chiloven.pulselink.transport.DiscoveredDevice
import top.chiloven.pulselink.transport.ScanFilter
import top.chiloven.pulselink.transport.TransportDeviceId
import top.chiloven.pulselink.transport.TransportException
import kotlin.time.Duration
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.withTimeoutOrNull

internal class KableScanner(
    private val advertisements: MutableMap<TransportDeviceId, Advertisement>,
) {

    suspend fun scan(
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
            throw TransportException.ScanFailed(
                error.message ?: "scan failed",
                error
            )
        }
        return found.values.toList()
    }

}
