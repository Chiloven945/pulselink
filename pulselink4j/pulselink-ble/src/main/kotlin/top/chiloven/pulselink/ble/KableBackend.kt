package top.chiloven.pulselink.ble

import com.juul.kable.Advertisement
import com.juul.kable.Peripheral
import top.chiloven.pulselink.ble.internal.KableConnector
import top.chiloven.pulselink.ble.internal.KableScanner
import top.chiloven.pulselink.transport.*
import java.util.concurrent.ConcurrentHashMap
import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel

public class KableBackend(
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO),
    private val connectTimeout: Duration = 15.seconds,
    private val operationTimeout: Duration = 5.seconds,
) : Backend, AutoCloseable {

    private val advertisements = ConcurrentHashMap<TransportDeviceId, Advertisement>()
    private val peripherals = ConcurrentHashMap<TransportDeviceId, Peripheral>()
    private val scanner = KableScanner(advertisements)
    private val connector = KableConnector(
        advertisements = advertisements,
        peripherals = peripherals,
        connectTimeout = connectTimeout,
        operationTimeout = operationTimeout,
    )

    override suspend fun scan(
        filter: ScanFilter,
        timeout: Duration
    ): List<DiscoveredDevice> =
        scanner.scan(filter, timeout)

    override suspend fun connect(device: DiscoveredDevice): GattLink =
        connector.connect(device)

    override suspend fun reconnect(target: ReconnectTarget): GattLink =
        connector.reconnect(target) { filter, timeout -> scanner.scan(filter, timeout) }

    override fun close() {
        connector.close()
        advertisements.clear()
        scope.cancel()
    }

}
