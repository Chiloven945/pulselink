package top.chiloven.pulselink.ble

import com.juul.kable.WriteType
import top.chiloven.pulselink.transport.CharacteristicInfo
import top.chiloven.pulselink.transport.GattCharacteristic
import top.chiloven.pulselink.transport.TransportException
import top.chiloven.pulselink.transport.WriteMode
import java.util.*
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class KableBleTest {

    private val service = UUID.fromString("0000180c-0000-1000-8000-00805f9b34fb")
    private val characteristic = GattCharacteristic(
        service,
        UUID.fromString("0000150a-0000-1000-8000-00805f9b34fb"),
    )
    private val info = CharacteristicInfo(
        id = characteristic,
        readable = false,
        writable = true,
        writableWithoutResponse = true,
        notifiable = false,
        indicatable = false,
    )

    @Test
    fun autoPrefersWithoutResponse() {
        assertEquals(
            WriteType.WithoutResponse,
            selectWriteType(info, WriteMode.Auto)
        )
    }

    @Test
    fun explicitModesRequireMatchingProperties() {
        assertEquals(
            WriteType.WithResponse,
            selectWriteType(info, WriteMode.WithResponse)
        )
        assertEquals(
            WriteType.WithoutResponse,
            selectWriteType(info, WriteMode.WithoutResponse)
        )
        assertFailsWith<TransportException.GattFailed> {
            selectWriteType(
                info.copy(writable = false),
                WriteMode.WithResponse
            )
        }
        assertFailsWith<TransportException.GattFailed> {
            selectWriteType(
                info.copy(writableWithoutResponse = false, writable = false),
                WriteMode.Auto
            )
        }
    }

    @Test
    fun uuidRoundTrips() {
        assertEquals(
            service,
            service.toKableUuid().toJavaUuid()
        )
        assertEquals(
            characteristic,
            characteristic.toKable().let {
                GattCharacteristic(
                    it.serviceUuid.toJavaUuid(),
                    it.characteristicUuid.toJavaUuid()
                )
            }
        )
    }

}
