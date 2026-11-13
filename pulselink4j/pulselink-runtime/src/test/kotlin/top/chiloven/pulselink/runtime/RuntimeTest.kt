package top.chiloven.pulselink.runtime

import top.chiloven.pulselink.core.Channel
import top.chiloven.pulselink.core.ConnectionState
import top.chiloven.pulselink.core.Intensity
import top.chiloven.pulselink.protocol.coyote.v3.CHAR_BATTERY
import top.chiloven.pulselink.protocol.coyote.v3.CHAR_NOTIFY
import top.chiloven.pulselink.protocol.coyote.v3.SERVICE_BATTERY
import top.chiloven.pulselink.protocol.coyote.v3.SERVICE_CONTROL
import top.chiloven.pulselink.testing.MockBackend
import top.chiloven.pulselink.testing.MockGattLink
import top.chiloven.pulselink.transport.DiscoveredDevice
import top.chiloven.pulselink.transport.GattCharacteristic
import top.chiloven.pulselink.transport.ScanFilter
import top.chiloven.pulselink.transport.TransportDeviceId
import java.util.*
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout

class RuntimeTest {

    private val device = DiscoveredDevice(
        id = TransportDeviceId("dev"),
        name = "47L121000",
        rssi = -40,
        serviceUuids = listOf(UUID.fromString("0000180c-0000-1000-8000-00805f9b34fb")),
    )

    @Test
    fun initializesAndAcknowledgesIntensity() = runBlocking {
        val link = MockGattLink(device.id)
        link.setReadData(
            GattCharacteristic(
                SERVICE_BATTERY,
                CHAR_BATTERY
            ),
            byteArrayOf(80),
        )
        val backend = MockBackend(listOf(device))
        backend.pushLink(link)
        val client = Client(backend, ClientConfig(connectTimeout = 2.seconds))
        val session = client.connectCoyoteV3(device)
        assertEquals(
            80,
            session.state.value.common.battery.valueOrNull()?.level?.value
        )
        assertEquals(
            ConnectionState.Ready,
            session.state.value.common.connection
        )
        assertEquals(
            1,
            link.writes().size
        )

        val job = launch {
            withTimeout(2.seconds) {
                while (
                    link.writes().none {
                        it.data.isNotEmpty() && it.data[0].toInt() and 0xFF == 0xB0
                    }
                ) {
                    delay(10.milliseconds)
                }
            }
            delay(50.milliseconds)
            val b0 = link.writes().first {
                it.data.isNotEmpty() && it.data[0].toInt() and 0xFF == 0xB0
            }
            val sequence = (b0.data[1].toInt() shr 4) and 0x0F
            link.injectNotification(
                GattCharacteristic(
                    SERVICE_CONTROL,
                    CHAR_NOTIFY
                ),
                byteArrayOf(0xB1.toByte(), sequence.toByte(), 10, 20),
            )
        }
        withTimeout(2.seconds) {
            session.setIntensity(Channel.A, Intensity.of(10))
        }
        job.join()
        assertEquals(
            Intensity.of(10),
            session.state.value.common.channel(Channel.A).intensity.valueOrNull()
        )
        session.close()
    }

    @Test
    fun scanDelegatesToBackend() = runBlocking {
        val backend = MockBackend(listOf(device))
        val client = Client(backend)
        val result = client.scan(ScanFilter.all(), 1.seconds)
        assertEquals(1, result.size)
        assertEquals(1, backend.scans.value)
        client.close()
    }

}
