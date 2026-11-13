package top.chiloven.pulselink.device

import top.chiloven.pulselink.device.internal.SessionFactory
import top.chiloven.pulselink.runtime.CoyoteV3Config
import top.chiloven.pulselink.transport.Backend
import top.chiloven.pulselink.transport.DiscoveredDevice
import top.chiloven.pulselink.transport.ScanFilter
import kotlin.time.Duration
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import top.chiloven.pulselink.runtime.Client as RuntimeClient

/** Connects and classifies local DG-LAB devices. */
public class DeviceClient(
    private val backend: Backend,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO),
) : AutoCloseable {

    private val runtime = RuntimeClient(backend)
    private val factory = SessionFactory(backend, runtime)
    private val sessions = mutableListOf<LocalSession>()

    /** Scans for devices. */
    public suspend fun scan(
        filter: ScanFilter = ScanFilter.all(),
        timeout: Duration,
    ): List<DiscoveredDevice> = backend.scan(filter, timeout)

    /** Scans by profile, retrying with service UUIDs when the name scan is empty. */
    public suspend fun scanProfile(
        profile: DeviceProfile,
        timeout: Duration,
    ): List<DiscoveredDevice> {
        val named = backend.scan(profile.scanFilter(), timeout)
                .filter(profile::matches)
        return named.ifEmpty {
            backend.scan(profile.fallbackScanFilter(), timeout)
                    .filter(profile::matches)
        }
    }

    /** Connects and initializes a local session. */
    public suspend fun connect(device: DiscoveredDevice): LocalSession {
        val session = factory.connect(device)
        sessions += session
        return session
    }

    /** Connects a Coyote V3 session. */
    public suspend fun connectCoyoteV3(
        device: DiscoveredDevice,
        config: CoyoteV3Config = CoyoteV3Config(),
    ): LocalSession.CoyoteV3 {
        val session = factory.connectCoyoteV3(device, config)
        sessions += session
        return session
    }

    override fun close() {
        sessions.forEach { it.close() }
        sessions.clear()
        runtime.close()
        scope.cancel()
    }

}
