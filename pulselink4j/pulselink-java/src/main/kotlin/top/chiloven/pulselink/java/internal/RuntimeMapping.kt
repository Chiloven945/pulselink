package top.chiloven.pulselink.java.internal

import top.chiloven.pulselink.core.ChannelMap
import top.chiloven.pulselink.core.Intensity
import top.chiloven.pulselink.core.IntensityDelta
import top.chiloven.pulselink.java.device.StopResult
import top.chiloven.pulselink.java.runtime.*
import top.chiloven.pulselink.protocol.coyote.v3.B1Message
import top.chiloven.pulselink.runtime.StopOutcome
import kotlin.time.toJavaDuration
import kotlin.time.toKotlinDuration
import top.chiloven.pulselink.runtime.CoyoteV3Balance as KotlinCoyoteV3Balance
import top.chiloven.pulselink.runtime.CoyoteV3Config as KotlinCoyoteV3Config
import top.chiloven.pulselink.runtime.CoyoteV3Event as KotlinCoyoteV3Event
import top.chiloven.pulselink.runtime.CoyoteV3Settings as KotlinCoyoteV3Settings
import top.chiloven.pulselink.runtime.CoyoteV3State as KotlinCoyoteV3State
import top.chiloven.pulselink.runtime.PatternPlaybackState as KotlinPatternPlaybackState
import top.chiloven.pulselink.runtime.PlayEndReason as KotlinPlayEndReason
import top.chiloven.pulselink.runtime.ReconnectPolicy as KotlinReconnectPolicy
import top.chiloven.pulselink.runtime.SafetyPolicy as KotlinSafetyPolicy

internal fun CoyoteV3Config.toKotlin(): KotlinCoyoteV3Config =
    KotlinCoyoteV3Config(
        ackTimeout = ackTimeout().toKotlinDuration(),
        tickInterval = tickInterval().toKotlinDuration(),
        lagThreshold = lagThreshold().toKotlinDuration(),
        safety = safety().toKotlin(),
        reconnect = reconnect().toKotlin(),
        initialSoftLimit = ChannelMap(
            Intensity.of(initialSoftLimitA()),
            Intensity.of(initialSoftLimitB()),
        ),
        balance = balance()?.toKotlin(),
    )

internal fun SafetyPolicy.toKotlin(): KotlinSafetyPolicy =
    KotlinSafetyPolicy(
        maxIntensity = ChannelMap(
            Intensity.of(maxIntensityA()),
            Intensity.of(maxIntensityB()),
        ),
        maxDelta = if (maxDeltaA() != null && maxDeltaB() != null) {
            ChannelMap(
                IntensityDelta.of(maxDeltaA()!!),
                IntensityDelta.of(maxDeltaB()!!),
            )
        } else {
            null
        },
        allowTemporary = allowTemporary(),
        requireSoftLimit = requireSoftLimit(),
    )

internal fun ReconnectPolicy.toKotlin(): KotlinReconnectPolicy =
    KotlinReconnectPolicy(
        enabled = enabled(),
        maxAttempts = maxAttempts(),
        initialBackoff = initialBackoff().toKotlinDuration(),
        maxBackoff = maxBackoff().toKotlinDuration(),
        jitter = jitter(),
    )

internal fun CoyoteV3Balance.toKotlin(): KotlinCoyoteV3Balance =
    KotlinCoyoteV3Balance(
        frequency = ChannelMap(frequencyA(), frequencyB()),
        intensity = ChannelMap(intensityA(), intensityB()),
    )

internal fun KotlinCoyoteV3State.toJava(): CoyoteV3State =
    CoyoteV3State(
        common.toJava(),
        settings.valueOrNull()?.toJava(),
        pattern?.toJava(),
        lastB1?.toJava(),
    )

private fun KotlinCoyoteV3Settings.toJava(): CoyoteV3Settings =
    CoyoteV3Settings(
        softLimit.a.value,
        softLimit.b.value,
        frequencyBalance.a,
        frequencyBalance.b,
        intensityBalance.a,
        intensityBalance.b,
    )

private fun KotlinPatternPlaybackState.toJava(): PatternPlaybackState =
    PatternPlaybackState(
        id.value,
        elapsed.toJavaDuration(),
        frameIndex,
        looped,
    )

private fun B1Message.toJava(): B1Snapshot =
    B1Snapshot(
        sequence.value,
        intensity.a.value,
        intensity.b.value,
    )

internal fun KotlinCoyoteV3Event.toJava(): CoyoteV3Event =
    when (this) {
        is KotlinCoyoteV3Event.Device ->
            CoyoteV3Event.Device(event.toJava())

        is KotlinCoyoteV3Event.Reconnecting ->
            CoyoteV3Event.Reconnecting(attempt, delay.toJavaDuration())

        KotlinCoyoteV3Event.Reconnected ->
            CoyoteV3Event.Reconnected()

        is KotlinCoyoteV3Event.ReconnectFailed ->
            CoyoteV3Event.ReconnectFailed(attempts, error)

        is KotlinCoyoteV3Event.PatternStarted ->
            CoyoteV3Event.PatternStarted(id.value)

        is KotlinCoyoteV3Event.PatternFinished ->
            CoyoteV3Event.PatternFinished(id.value, reason.toJava())

        is KotlinCoyoteV3Event.SchedulerLag ->
            CoyoteV3Event.SchedulerLag(behind.toJavaDuration())

        is KotlinCoyoteV3Event.AckTimeout ->
            CoyoteV3Event.AckTimeout(sequence.value)

        is KotlinCoyoteV3Event.B1Mismatch ->
            CoyoteV3Event.B1Mismatch(expected, received)

        KotlinCoyoteV3Event.Closed ->
            CoyoteV3Event.Closed()
    }

internal fun KotlinPlayEndReason.toJava(): PlayEndReason =
    when (this) {
        KotlinPlayEndReason.Completed -> PlayEndReason.COMPLETED
        KotlinPlayEndReason.Stopped -> PlayEndReason.STOPPED
        KotlinPlayEndReason.Replaced -> PlayEndReason.REPLACED
        KotlinPlayEndReason.SchedulerLag -> PlayEndReason.SCHEDULER_LAG
        KotlinPlayEndReason.LinkLost -> PlayEndReason.LINK_LOST
        KotlinPlayEndReason.Shutdown -> PlayEndReason.SHUTDOWN
        KotlinPlayEndReason.Failed -> PlayEndReason.FAILED
    }

internal fun StopOutcome.toJava(): StopResult =
    StopResult(
        acknowledged,
        intensity.a.valueOrNull()?.value ?: 0,
        intensity.b.valueOrNull()?.value ?: 0,
    )
