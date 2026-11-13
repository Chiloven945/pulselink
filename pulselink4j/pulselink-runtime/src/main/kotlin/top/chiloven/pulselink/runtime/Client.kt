package top.chiloven.pulselink.runtime

import top.chiloven.pulselink.core.BleDeviceId
import top.chiloven.pulselink.core.DeviceId
import top.chiloven.pulselink.core.DeviceInfo
import top.chiloven.pulselink.core.DeviceKind
import top.chiloven.pulselink.transport.Backend
import top.chiloven.pulselink.transport.DiscoveredDevice
import top.chiloven.pulselink.transport.ScanFilter
import kotlin.time.Duration
import kotlinx.coroutines.*
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/** Creates local sessions over a transport backend. */
public class Client(
    private val backend: Backend,
    private val config: ClientConfig = ClientConfig(),
) : AutoCloseable {

    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val sessions = mutableListOf<CoyoteV3Session>()
    private val sessionsLock = Mutex()

    /** Scans for devices. */
    public suspend fun scan(
        filter: ScanFilter = ScanFilter.all(),
        timeout: Duration,
    ): List<DiscoveredDevice> = backend.scan(filter, timeout)

    /** Connects and initializes a Coyote V3 session. */
    public suspend fun connectCoyoteV3(
        device: DiscoveredDevice,
        config: CoyoteV3Config = CoyoteV3Config(),
    ): CoyoteV3Session {
        val link = withTimeout(this.config.connectTimeout) { backend.connect(device) }
        val session = CoyoteV3Session(
            backend = backend,
            link = link,
            info = DeviceInfo(
                DeviceId.Ble(BleDeviceId(device.id.value)),
                DeviceKind.CoyoteV3,
                device.name ?: "Coyote V3",
            ),
            config = config,
            fallbackFilter = ScanFilter(
                namePrefixes = listOf("47L121000"),
                serviceUuids = emptyList(),
            ),
        )
        session.initialize()
        sessionsLock.withLock { sessions += session }
        return session
    }

    override fun close() {
        scope.cancel()
    }

    /** Closes all sessions and the client scope. */
    public suspend fun shutdown() {
        val toClose = sessionsLock.withLock { sessions.toList().also { sessions.clear() } }
        toClose.forEach { it.close() }
        scope.cancel()
    }

}
