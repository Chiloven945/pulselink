package top.chiloven.pulselink.java

import top.chiloven.pulselink.java.core.Channel
import top.chiloven.pulselink.java.core.ConnectionState
import top.chiloven.pulselink.java.core.DeviceState
import top.chiloven.pulselink.java.device.CoyoteV2Session
import top.chiloven.pulselink.java.internal.BridgeContext
import top.chiloven.pulselink.java.internal.toKotlin
import top.chiloven.pulselink.java.runtime.CoyoteV3Config
import top.chiloven.pulselink.java.transport.DiscoveredDevice
import top.chiloven.pulselink.java.transport.PulseLinkProfiles
import top.chiloven.pulselink.protocol.coyote.v3.CHAR_BATTERY
import top.chiloven.pulselink.protocol.coyote.v3.CHAR_NOTIFY
import top.chiloven.pulselink.protocol.coyote.v3.SERVICE_BATTERY
import top.chiloven.pulselink.protocol.coyote.v3.SERVICE_CONTROL
import top.chiloven.pulselink.testing.MockBackend
import top.chiloven.pulselink.testing.MockGattLink
import top.chiloven.pulselink.transport.GattCharacteristic
import top.chiloven.pulselink.transport.TransportDeviceId
import java.time.Duration
import java.util.concurrent.CompletableFuture
import java.util.concurrent.Flow
import java.util.concurrent.TimeUnit
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlin.time.Duration.Companion.milliseconds
import kotlinx.coroutines.*

class JavaLocalFacadeTest {

    @Test
    fun connectsAndControlsCoyoteV3() =
        runBlocking<Unit> {
            val device = DiscoveredDevice(
                "dev-v3",
                "47L121000",
                -40,
                emptyList(),
                emptyList(),
                true,
            )
            val backend = MockBackend(listOf(device.toKotlin()))
            val link = MockGattLink(TransportDeviceId("dev-v3"))
            link.setReadData(
                GattCharacteristic(
                    SERVICE_BATTERY,
                    CHAR_BATTERY
                ),
                byteArrayOf(80),
            )
            backend.pushLink(link)

            val ackJob = launch(Dispatchers.Default) { acknowledgeB0(link) }
            try {
                testClient(backend).use { client ->
                    val scanned = client
                            .scanProfile(PulseLinkProfiles.COYOTE_V3, Duration.ofSeconds(1))
                            .get(5, TimeUnit.SECONDS)
                    assertEquals(1, scanned.size)

                    val session = client
                            .connectCoyoteV3(device, CoyoteV3Config.defaults())
                            .get(5, TimeUnit.SECONDS)

                    assertEquals(
                        ConnectionState.READY,
                        session.state().connection()
                    )
                    assertEquals(
                        80,
                        session.state().battery()?.level()
                    )
                    assertEquals(
                        ConnectionState.READY,
                        firstState(session.states())
                                .get(5, TimeUnit.SECONDS)
                                .connection(),
                    )

                    session
                            .setIntensity(Channel.A, 10)
                            .get(5, TimeUnit.SECONDS)
                    assertTrue(
                        link.writes().any { write ->
                            write.data.size == 20 &&
                                    (write.data[0].toInt() and 0xFF) == 0xB0
                        }
                    )

                    val stopped = session
                            .stop()
                            .get(5, TimeUnit.SECONDS)
                    assertTrue(stopped.acknowledged())

                    session
                            .shutdown()
                            .get(5, TimeUnit.SECONDS)
                }
            } finally {
                ackJob.cancelAndJoin()
            }
        }

    @Test
    fun connectsAndControlsCoyoteV2() =
        runBlocking<Unit> {
            val device = DiscoveredDevice(
                "dev-v2",
                "D-LAB ESTIM01",
                -40,
                emptyList(),
                emptyList(),
                true,
            )
            val backend = MockBackend(listOf(device.toKotlin()))
            val link = MockGattLink(TransportDeviceId("dev-v2"))
            backend.pushLink(link)

            testClient(backend).use { client ->
                val session = client
                        .connect(device)
                        .get(5, TimeUnit.SECONDS) as CoyoteV2Session

                session
                        .setIntensity(Channel.A, 50)
                        .get(5, TimeUnit.SECONDS)
                assertTrue(link.writes().any { it.data.size == 3 })
                session
                        .shutdown()
                        .get(5, TimeUnit.SECONDS)
            }
        }

    private fun testClient(backend: MockBackend): PulseLinkClient =
        PulseLinkClient(
            PulseLinkBackendAdapter(
                BridgeContext(),
                backend
            )
        )

    private fun firstState(
        publisher: Flow.Publisher<DeviceState>
    ): CompletableFuture<DeviceState> =
        CompletableFuture<DeviceState>().apply {
            publisher.subscribe(object : Flow.Subscriber<DeviceState> {
                override fun onSubscribe(subscription: Flow.Subscription) {
                    subscription.request(1)
                }

                override fun onNext(item: DeviceState) {
                    complete(item)
                }

                override fun onError(error: Throwable) {
                    completeExceptionally(error)
                }

                override fun onComplete() {
                    completeExceptionally(
                        IllegalStateException("state publisher completed")
                    )
                }
            })
        }

    private suspend fun acknowledgeB0(link: MockGattLink) {
        var seen = 0
        while (currentCoroutineContext().isActive) {
            val writes = link.writes()
            while (seen < writes.size) {
                val data = writes[seen].data
                if (data.size == 20 && (data[0].toInt() and 0xFF) == 0xB0) {
                    val sequence = (data[1].toInt() shr 4) and 0x0F
                    link.injectNotification(
                        GattCharacteristic(
                            SERVICE_CONTROL,
                            CHAR_NOTIFY
                        ),
                        byteArrayOf(
                            0xB1.toByte(),
                            sequence.toByte(),
                            0,
                            0
                        ),
                    )
                }
                seen += 1
            }
            delay(2.milliseconds)
        }
    }

}
