package top.chiloven.pulselink.device

import top.chiloven.pulselink.core.DeviceKind
import top.chiloven.pulselink.transport.DiscoveredDevice
import top.chiloven.pulselink.transport.TransportDeviceId
import java.util.*
import kotlin.test.Test
import kotlin.test.assertEquals

class ProfileTest {

    private val control = UUID.fromString("0000180c-0000-1000-8000-00805f9b34fb")

    private fun device(
        name: String?,
        services: List<UUID> = listOf(control)
    ) =
        DiscoveredDevice(
            TransportDeviceId("dev"),
            name,
            null,
            services,
        )

    @Test
    fun classifiesNamedDevices() {
        assertEquals(
            DeviceKind.CoyoteV3,
            classify(device("47L121000"))
        )
        assertEquals(
            DeviceKind.Opossum,
            classify(device("47L127000"))
        )
        assertEquals(
            DeviceKind.Civet,
            classify(device("47L124000"))
        )
        assertEquals(
            DeviceKind.PawPrints,
            classify(device("47L120100"))
        )
        assertEquals(
            DeviceKind.CoyoteV2,
            classify(
                device(
                    "D-LAB ESTIM01",
                    listOf(UUID.fromString("0000180b-0000-1000-8000-00805f9b34fb"))
                )
            )
        )
    }

    @Test
    fun unnamedControlServiceIsV3Candidate() {
        assertEquals(
            DeviceKind.CoyoteV3,
            classify(device(null))
        )
        assertEquals(
            true,
            COYOTE_V3.matches(device(null))
        )
        assertEquals(
            false,
            OPOSSUM.matches(device(null))
        )
        assertEquals(
            false,
            COYOTE_V3.matches(device("47L124000"))
        )
    }

    @Test
    fun profileFallbackFilters() {
        assertEquals(
            listOf("47L121000"),
            COYOTE_V3.scanFilter().namePrefixes
        )
        assertEquals(
            listOf(control),
            COYOTE_V3.fallbackScanFilter().serviceUuids
        )
    }

}
