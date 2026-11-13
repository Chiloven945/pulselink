package top.chiloven.pulselink.java

import top.chiloven.pulselink.ble.KableBackend
import top.chiloven.pulselink.device.DeviceClient
import top.chiloven.pulselink.java.device.CoyoteV3Session
import top.chiloven.pulselink.java.device.LocalSession
import top.chiloven.pulselink.java.internal.BridgeContext
import top.chiloven.pulselink.java.internal.CoyoteV3SessionAdapter
import top.chiloven.pulselink.java.internal.toJava
import top.chiloven.pulselink.java.internal.toKotlin
import top.chiloven.pulselink.java.runtime.CoyoteV3Config
import top.chiloven.pulselink.java.transport.DeviceProfile
import top.chiloven.pulselink.java.transport.DiscoveredDevice
import top.chiloven.pulselink.java.transport.ScanFilter
import top.chiloven.pulselink.transport.Backend
import java.time.Duration
import java.util.concurrent.CompletableFuture
import java.util.concurrent.CopyOnWriteArrayList
import kotlin.time.toKotlinDuration

internal class PulseLinkBackendAdapter(
    private val context: BridgeContext,
    private val backend: Backend,
) : PulseLinkBackend {

    constructor(context: BridgeContext) : this(context, KableBackend())

    private val client = DeviceClient(backend)
    private val sessions = CopyOnWriteArrayList<LocalSession>()

    override fun scan(
        filter: ScanFilter,
        timeout: Duration,
    ): CompletableFuture<List<DiscoveredDevice>> =
        context.future {
            client.scan(filter.toKotlin(), timeout.toKotlinDuration())
                    .map { it.toJava() }
        }

    override fun scanProfile(
        profile: DeviceProfile,
        timeout: Duration,
    ): CompletableFuture<List<DiscoveredDevice>> =
        context.future {
            client.scanProfile(profile.toKotlin(), timeout.toKotlinDuration())
                    .map { it.toJava() }
        }

    override fun connect(device: DiscoveredDevice): CompletableFuture<LocalSession> =
        context.future {
            client.connect(device.toKotlin()).toJava().also { sessions += it }
        }

    override fun connectCoyoteV3(
        device: DiscoveredDevice,
        config: CoyoteV3Config,
    ): CompletableFuture<CoyoteV3Session> =
        context.future {
            val session = client.connectCoyoteV3(
                device.toKotlin(),
                config.toKotlin(),
            )
            CoyoteV3SessionAdapter(
                BridgeContext(),
                session.runtime
            ).also { sessions += it }
        }

    override fun shutdown(): CompletableFuture<Void> =
        context.futureVoid {
            closeSessions()
            client.close()
            closeBackend()
        }.whenComplete { _, _ -> context.close() }

    override fun close() {
        closeSessions()
        runCatching { client.close() }
        closeBackend()
        context.close()
    }

    private fun closeSessions() {
        sessions.forEach { runCatching { it.close() } }
        sessions.clear()
    }

    private fun closeBackend() {
        runCatching { (backend as? AutoCloseable)?.close() }
    }

}
