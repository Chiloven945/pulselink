package top.chiloven.pulselink.java.internal

import top.chiloven.pulselink.java.pattern.ChannelFrameSpec
import top.chiloven.pulselink.java.pattern.Pattern
import top.chiloven.pulselink.java.pattern.PulseSample
import top.chiloven.pulselink.java.pattern.StereoFrameSpec
import top.chiloven.pulselink.java.runtime.PlayOptions
import top.chiloven.pulselink.pattern.*
import top.chiloven.pulselink.pattern.Pattern as KotlinPattern
import top.chiloven.pulselink.pattern.PulseSample as KotlinPulseSample
import top.chiloven.pulselink.runtime.PlayOptions as KotlinPlayOptions

internal fun Pattern.toKotlin(): KotlinPattern =
    KotlinPattern.of(
        Timeline.of(frames().map { it.toKotlin() }),
        looped(),
    )

internal fun PlayOptions.toKotlin(): KotlinPlayOptions =
    KotlinPlayOptions(replace())

private fun StereoFrameSpec.toKotlin(): StereoFrame =
    StereoFrame(
        a().toKotlin(),
        b().toKotlin()
    )

private fun ChannelFrameSpec.toKotlin(): ChannelFrame =
    when (this) {
        is ChannelFrameSpec.Active ->
            ChannelFrame.Active(
                samples().map { it.toKotlin() }
            )

        is ChannelFrameSpec.Silent ->
            ChannelFrame.Silent
    }

private fun PulseSample.toKotlin(): KotlinPulseSample =
    KotlinPulseSample(
        PulseInterval.of(intervalMillis()),
        PulseIntensity.of(intensity()),
    )
