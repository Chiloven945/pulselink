package top.chiloven.pulselink.java.internal

import top.chiloven.pulselink.core.Intensity
import top.chiloven.pulselink.java.core.Channel
import top.chiloven.pulselink.java.core.DeviceKind
import top.chiloven.pulselink.java.core.DeviceState
import top.chiloven.pulselink.java.core.LocalEvent
import top.chiloven.pulselink.java.device.CoyoteV2Session
import top.chiloven.pulselink.pattern.PulseInterval
import top.chiloven.pulselink.protocol.coyote.v2.PwmPulse
import java.util.concurrent.CompletableFuture
import java.util.concurrent.Flow
import kotlinx.coroutines.flow.map
import top.chiloven.pulselink.device.LocalSession as KotlinLocalSession

internal class CoyoteV2SessionAdapter(
    private val context: BridgeContext,
    private val session: KotlinLocalSession.CoyoteV2,
) : CoyoteV2Session {

    override fun kind(): DeviceKind =
        DeviceKind.CoyoteV2()

    override fun state(): DeviceState =
        session.state.value.toJava()

    override fun states(): Flow.Publisher<DeviceState> =
        context.publisher(
            session.state.map { it.toJava() },
            conflate = true
        )

    override fun events(): Flow.Publisher<LocalEvent> =
        context.publisher(
            session.events.map { it.toJava() },
            conflate = false
        )

    override fun setIntensity(channel: Channel, intensity: Int): CompletableFuture<Void> =
        context.futureVoid {
            session.setIntensity(
                channel.toKotlin(),
                Intensity.of(intensity)
            )
        }

    override fun setPulse(
        channel: Channel,
        x: Int,
        y: Int,
        z: Int
    ): CompletableFuture<Void> =
        context.futureVoid {
            session.setPulse(
                channel.toKotlin(),
                PwmPulse(x, y, z)
            )
        }

    override fun setPulseInterval(channel: Channel, milliseconds: Int): CompletableFuture<Void> =
        context.futureVoid {
            session.setPulse(
                channel.toKotlin(),
                PulseInterval.of(milliseconds)
            )
        }

    override fun shutdown(): CompletableFuture<Void> =
        context
                .futureVoid { session.shutdown() }
                .whenComplete { _, _ -> context.close() }

    override fun close() {
        session.close()
        context.close()
    }

}
