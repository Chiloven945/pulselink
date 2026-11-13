package top.chiloven.pulselink.java.internal

import top.chiloven.pulselink.java.core.DeviceKind
import top.chiloven.pulselink.java.core.DeviceState
import top.chiloven.pulselink.java.core.LocalEvent
import top.chiloven.pulselink.java.device.CivetSession
import java.util.concurrent.CompletableFuture
import java.util.concurrent.Flow
import kotlinx.coroutines.flow.map
import top.chiloven.pulselink.device.LocalSession as KotlinLocalSession

internal class CivetSessionAdapter(
    private val context: BridgeContext,
    private val session: KotlinLocalSession.Civet,
) : CivetSession {

    override fun kind(): DeviceKind =
        DeviceKind.Civet()

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

    override fun startPressure(color: Int): CompletableFuture<Void> =
        context.futureVoid {
            session.startPressure(color)
        }

    override fun stopPressure(color: Int): CompletableFuture<Void> =
        context.futureVoid {
            session.stopPressure(color)
        }

    override fun resetPressure(): CompletableFuture<Void> =
        context.futureVoid {
            session.resetPressure()
        }

    override fun setOrientation(value: Int): CompletableFuture<Void> =
        context.futureVoid {
            session.setOrientation(value)
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
