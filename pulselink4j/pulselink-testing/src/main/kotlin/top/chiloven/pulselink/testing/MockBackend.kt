package top.chiloven.pulselink.testing

import top.chiloven.pulselink.transport.*
import java.util.*
import kotlin.time.Duration
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

/** A controllable in-memory [Backend]. */
public class MockBackend(
    private val devices: List<DiscoveredDevice> = emptyList(),
) : Backend {

    private val links = ArrayDeque<MockGattLink>()
    private val scanCount = MutableStateFlow(0)
    private val reconnectCount = MutableStateFlow(0)
    private var last: MockGattLink? = null

    /** Number of scan calls. */
    public val scans: StateFlow<Int> = scanCount.asStateFlow()

    /** Number of reconnect calls. */
    public val reconnects: StateFlow<Int> = reconnectCount.asStateFlow()

    /** Queues a link for the next connect/reconnect. */
    public fun pushLink(link: MockGattLink) {
        links.addLast(link)
    }

    /** Returns the most recently returned link. */
    public fun lastLink(): MockGattLink? =
        last

    override suspend fun scan(
        filter: ScanFilter,
        timeout: Duration
    ): List<DiscoveredDevice> {
        scanCount.value += 1
        return devices.filter(filter::matches)
    }

    override suspend fun connect(device: DiscoveredDevice): GattLink {
        val link = links.pollFirst()
            ?: throw TransportException.ConnectFailed("no mock link queued")
        last = link
        return link
    }

    override suspend fun reconnect(target: ReconnectTarget): GattLink {
        reconnectCount.value += 1
        val link = links.pollFirst()
            ?: throw TransportException.ConnectFailed("no mock link queued")
        last = link
        return link
    }

}
