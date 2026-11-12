package top.chiloven.pulselink.ble

import org.junit.jupiter.api.condition.EnabledIfEnvironmentVariable
import top.chiloven.pulselink.transport.ScanFilter
import kotlin.test.Test
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.runBlocking

/** Opt-in hardware smoke test; run with PULSELINK_HW=1. */
@EnabledIfEnvironmentVariable(named = "PULSELINK_HW", matches = "1")
class KableHardwareTest {

    @Test
    fun scansForDevices() = runBlocking {
        KableBackend().use { backend ->
            val devices = backend.scan(ScanFilter.all(), 5.seconds)
            println("discovered ${devices.size} BLE device(s)")
            devices.forEach {
                println("${it.id.value} name=${it.name} rssi=${it.rssi}")
            }
        }
    }

}
