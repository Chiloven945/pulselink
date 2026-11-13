package top.chiloven.pulselink.java.internal

import top.chiloven.pulselink.core.Intensity
import top.chiloven.pulselink.java.core.Channel
import top.chiloven.pulselink.java.core.DeviceKind
import top.chiloven.pulselink.java.core.DeviceState
import top.chiloven.pulselink.java.core.LocalEvent
import top.chiloven.pulselink.java.device.OpossumSession
import java.util.concurrent.CompletableFuture
import java.util.concurrent.Flow
import kotlinx.coroutines.flow.map
import top.chiloven.pulselink.device.LocalSession as KotlinLocalSession

internal class OpossumSessionAdapter(
    private val context: BridgeContext,
    private val session: KotlinLocalSession.Opossum,
) : OpossumSession {

    override fun kind(): DeviceKind =
        DeviceKind.Opossum()

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

    override fun setIntensity(
        channel: Channel,
        intensity: Int
    ): CompletableFuture<Void> =
        context.futureVoid {
            session.setIntensity(
                channel.toKotlin(),
                Intensity.of(intensity)
            )
        }

    override fun pulse(
        channel: Channel,
        values: IntArray
    ): CompletableFuture<Void> {
        require(values.size == 4) { "Opossum pulse requires exactly 4 values" }
        return context.futureVoid {
            session.pulse(channel.toKotlin(), values.toList())
        }
    }

    override fun display(a: Int, b: Int): CompletableFuture<Void> =
        context.futureVoid {
            session.display(
                Intensity.of(a),
                Intensity.of(b)
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
