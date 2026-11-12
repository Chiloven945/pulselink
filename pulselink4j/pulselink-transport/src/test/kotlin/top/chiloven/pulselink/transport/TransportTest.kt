package top.chiloven.pulselink.transport

import java.util.*
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class TransportTest {

    private val service = UUID.fromString("0000180c-0000-1000-8000-00805f9b34fb")

    private fun device(name: String?, services: List<UUID> = emptyList()) =
        DiscoveredDevice(
            id = TransportDeviceId("dev"),
            name = name,
            rssi = -40,
            serviceUuids = services,
        )

    @Test
    fun emptyFilterMatchesEverything() {
        assertTrue(ScanFilter.all().matches(device("anything")))
    }

    @Test
    fun nameOrServiceMatches() {
        val filter = ScanFilter(namePrefixes = listOf("47L"), serviceUuids = listOf(service))
        assertTrue(filter.matches(device("47L121000")))
        assertTrue(filter.matches(device(null, listOf(service))))
        assertFalse(filter.matches(device("Other")))
    }

    @Test
    fun gattCharacteristicCarriesService() {
        val id = GattCharacteristic(
            service,
            UUID.fromString("0000150a-0000-1000-8000-00805f9b34fb")
        )
        assertEquals(service, id.service)
    }

}
