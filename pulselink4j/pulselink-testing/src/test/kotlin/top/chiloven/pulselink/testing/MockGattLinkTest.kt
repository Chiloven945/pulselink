package top.chiloven.pulselink.testing

import top.chiloven.pulselink.transport.*
import java.util.*
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.runTest

class MockGattLinkTest {

    private val service = UUID.fromString("0000180c-0000-1000-8000-00805f9b34fb")
    private val char = GattCharacteristic(
        service,
        UUID.fromString("0000150a-0000-1000-8000-00805f9b34fb")
    )
    private val info = CharacteristicInfo(
        char,
        false,
        true,
        true,
        false,
        false
    )

    @Test
    fun writeReadSubscribeAndDisconnect() = runTest {
        val link = MockGattLink(
            TransportDeviceId("dev"),
            mapOf(char to info)
        )
        link.setReadData(
            char,
            byteArrayOf(1, 2, 3)
        )
        link.write(
            char,
            byteArrayOf(4),
            WriteMode.Auto
        )
        assertEquals(1, link.writes().size)
        assertEquals(
            listOf<Byte>(1, 2, 3),
            link.read(char).toList()
        )
        assertEquals(
            info,
            link.characteristic(char)
        )

        val received = mutableListOf<ByteArray>()
        val job = launch(start = CoroutineStart.UNDISPATCHED) {
            link.subscribe(char)
                    .first()
                    .let(received::add)
        }
        link.injectNotification(char, byteArrayOf(9))
        job.join()
        assertEquals(
            listOf<Byte>(9),
            received.single().toList()
        )

        link.disconnect()
        assertFalse(link.isConnected)
        assertFailsWith<TransportException.Disconnected> {
            link.read(char)
        }
    }

}
