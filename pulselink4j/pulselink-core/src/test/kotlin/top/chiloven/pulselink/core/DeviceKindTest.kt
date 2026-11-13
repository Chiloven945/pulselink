package top.chiloven.pulselink.core

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class DeviceKindTest {

    @Test
    fun deviceKindRemoteMapping() {
        assertEquals(
            DeviceKind.CoyoteV2,
            DeviceKind.fromRemoteType("COYOTE_020")
        )
        assertEquals(
            DeviceKind.CoyoteV3,
            DeviceKind.fromRemoteType("COYOTE_030")
        )
        assertEquals(
            DeviceKind.Civet,
            DeviceKind.fromRemoteType("BMTR_1")
        )
        assertEquals(
            DeviceKind.Opossum,
            DeviceKind.fromRemoteType("OVC_1")
        )
        assertEquals(
            DeviceKind.Unknown("X"),
            DeviceKind.fromRemoteType("X")
        )
        assertEquals(
            "COYOTE_030",
            DeviceKind.CoyoteV3.remoteType
        )
        assertNull(DeviceKind.PawPrints.remoteType)
    }

}
