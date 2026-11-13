package top.chiloven.pulselink.testing

import top.chiloven.pulselink.transport.DiscoveredDevice
import top.chiloven.pulselink.transport.ScanFilter
import top.chiloven.pulselink.transport.TransportDeviceId
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.test.runTest

class MockBackendTest {

    @Test
    fun backendScanConnectReconnect() = runTest {
        val device = DiscoveredDevice(
            TransportDeviceId("dev"),
            "47L",
            -40,
            emptyList()
        )
        val backend = MockBackend(listOf(device))
        val link = MockGattLink(device.id)
        backend.pushLink(link)
        assertEquals(
            listOf(device),
            backend.scan(ScanFilter.all(), 1.seconds)
        )
        backend.connect(device)
        assertEquals(link, backend.lastLink())
        assertEquals(1, backend.scans.value)
    }

}
