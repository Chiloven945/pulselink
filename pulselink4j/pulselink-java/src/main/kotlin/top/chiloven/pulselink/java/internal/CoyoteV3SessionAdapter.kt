package top.chiloven.pulselink.java.internal

import top.chiloven.pulselink.core.Intensity
import top.chiloven.pulselink.core.IntensityDelta
import top.chiloven.pulselink.java.core.Channel
import top.chiloven.pulselink.java.core.DeviceKind
import top.chiloven.pulselink.java.core.DeviceState
import top.chiloven.pulselink.java.core.LocalEvent
import top.chiloven.pulselink.java.device.CoyoteV3Session
import top.chiloven.pulselink.java.device.PlayHandle
import top.chiloven.pulselink.java.device.StopResult
import top.chiloven.pulselink.java.pattern.Pattern
import top.chiloven.pulselink.java.runtime.*
import java.util.concurrent.CompletableFuture
import java.util.concurrent.Flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapNotNull
import top.chiloven.pulselink.runtime.CoyoteV3Event as KotlinCoyoteV3Event
import top.chiloven.pulselink.runtime.CoyoteV3Session as KotlinCoyoteV3Session
import top.chiloven.pulselink.runtime.PlayHandle as KotlinPlayHandle

internal class CoyoteV3SessionAdapter(
    private val context: BridgeContext,
    private val session: KotlinCoyoteV3Session,
) : CoyoteV3Session {

    override fun kind(): DeviceKind =
        DeviceKind.CoyoteV3()

    override fun state(): DeviceState =
        session.state.value.common.toJava()

    override fun states(): Flow.Publisher<DeviceState> =
        context.publisher(
            session.state.map { it.common.toJava() },
            conflate = true,
        )

    override fun events(): Flow.Publisher<LocalEvent> =
        context.publisher(
            session.events.mapNotNull { event ->
                (event as? KotlinCoyoteV3Event.Device)?.event?.toLocalEvent()
            },
            conflate = false,
        )

    override fun runtimeState(): CoyoteV3State =
        session.state.value.toJava()

    override fun runtimeStates(): Flow.Publisher<CoyoteV3State> =
        context.publisher(
            session.state.map { it.toJava() },
            conflate = true
        )

    override fun runtimeEvents(): Flow.Publisher<CoyoteV3Event> =
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

    override fun adjustIntensity(channel: Channel, delta: Int): CompletableFuture<Void> =
        context.futureVoid {
            session.adjustIntensity(
                channel.toKotlin(),
                IntensityDelta.of(delta)
            )
        }

    override fun setSoftLimit(channel: Channel, intensity: Int): CompletableFuture<Void> =
        context.futureVoid {
            session.setSoftLimit(
                channel.toKotlin(),
                Intensity.of(intensity)
            )
        }

    override fun setBalance(balance: CoyoteV3Balance): CompletableFuture<Void> =
        context.futureVoid {
            session.setBalance(balance.toKotlin())
        }

    override fun play(pattern: Pattern): CompletableFuture<PlayHandle> =
        play(pattern, PlayOptions.defaults())

    override fun play(
        pattern: Pattern,
        options: PlayOptions
    ): CompletableFuture<PlayHandle> =
        context.future {
            val handle = session.play(
                pattern.toKotlin(),
                options.toKotlin()
            )
            PlayHandleAdapter(context, handle)
        }

    override fun stop(): CompletableFuture<StopResult> =
        context.future { session.stop().toJava() }

    override fun reconnect(): CompletableFuture<Void> =
        context.futureVoid { session.reconnect() }

    override fun shutdown(): CompletableFuture<Void> =
        context
                .futureVoid { session.close() }
                .whenComplete { _, _ -> context.close() }

    override fun close() {
        session.close()
        context.close()
    }

}

internal class PlayHandleAdapter(
    private val context: BridgeContext,
    private val handle: KotlinPlayHandle,
) : PlayHandle {

    override fun id(): Long =
        handle.id.value

    override fun isFinished(): Boolean =
        handle.isFinished

    override fun completion(): CompletableFuture<PlayEndReason> =
        context.future { handle.wait().toJava() }

    override fun stop(): CompletableFuture<PlayEndReason> =
        context.future { handle.stop().toJava() }

}
