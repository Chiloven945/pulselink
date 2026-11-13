package top.chiloven.pulselink.java.internal

import top.chiloven.pulselink.java.core.DeviceKind
import top.chiloven.pulselink.java.core.DeviceState
import top.chiloven.pulselink.java.core.LocalEvent
import top.chiloven.pulselink.java.device.PawPrintsProbability
import top.chiloven.pulselink.java.device.PawPrintsSession
import java.util.concurrent.CompletableFuture
import java.util.concurrent.Flow
import kotlinx.coroutines.flow.map
import top.chiloven.pulselink.device.LocalSession as KotlinLocalSession
import top.chiloven.pulselink.protocol.pawprints.Command50 as PawCommand50

internal class PawPrintsSessionAdapter(
    private val context: BridgeContext,
    private val session: KotlinLocalSession.PawPrints,
) : PawPrintsSession {

    override fun kind(): DeviceKind =
        DeviceKind.PawPrints()

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

    override fun noTrigger(color: Int): CompletableFuture<Void> =
        context.futureVoid {
            session.noTrigger(color)
        }

    override fun resetParameter(): CompletableFuture<Void> =
        context.futureVoid {
            session.resetParameter()
        }

    override fun autoDetectAngles(): CompletableFuture<Void> =
        context.futureVoid {
            session.autoDetectAngles()
        }

    override fun shoulderLight(color: Int): CompletableFuture<Void> =
        context.futureVoid {
            session.shoulderLight(color)
        }

    override fun shoulderLightFlash(
        colorA: Int,
        colorB: Int,
        speed: Int
    ): CompletableFuture<Void> =
        context.futureVoid {
            session.shoulderLightFlash(colorA, colorB, speed)
        }

    override fun configureRandomReaction(
        color: Int,
        eventId: Int,
        delayMin: Int,
        delayMax: Int,
        reactionSeconds: Int,
        increaseRate: Int,
        decreaseOnDeactivation: Int,
        decreaseRate: Int,
        increaseOnActivation: Int,
    ): CompletableFuture<Void> =
        context.futureVoid {
            session.configure(
                PawCommand50.randomReaction(
                    color = color,
                    eventId = eventId,
                    delayRange = delayMin..delayMax,
                    reactionSeconds = reactionSeconds,
                    increaseRate = increaseRate,
                    decreaseOnDeactivation = decreaseOnDeactivation,
                    decreaseRate = decreaseRate,
                    increaseOnActivation = increaseOnActivation,
                )
            )
        }

    override fun configureProbability(
        color: Int,
        events: List<PawPrintsProbability>,
        cooldownSeconds: Int,
    ): CompletableFuture<Void> =
        context.futureVoid {
            session.configure(
                PawCommand50.probability(
                    color = color,
                    events = events.map { it.eventId() to it.probability() },
                    cooldownSeconds = cooldownSeconds,
                )
            )
        }

    override fun configureExternalVoltage(
        color: Int,
        eventId: Int,
        highImpedance: Boolean,
        targetMin: Int,
        targetMax: Int,
        mappingRange: Int,
    ): CompletableFuture<Void> =
        context.futureVoid {
            session.configure(
                PawCommand50.externalVoltage(
                    color = color,
                    eventId = eventId,
                    highImpedance = highImpedance,
                    targetRange = targetMin to targetMax,
                    mappingRange = mappingRange,
                )
            )
        }

    override fun configureRawSensor(color: Int): CompletableFuture<Void> =
        context.futureVoid {
            session.configure(PawCommand50.rawSensor(color))
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
