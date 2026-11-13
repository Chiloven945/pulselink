package top.chiloven.pulselink.runtime.internal

import top.chiloven.pulselink.pattern.ChannelFrame
import top.chiloven.pulselink.protocol.coyote.v3.ChannelWaveform

internal fun ChannelFrame.toWave(): ChannelWaveform =
    when (this) {
        is ChannelFrame.Active -> ChannelWaveform.Active(samples)
        ChannelFrame.Silent -> ChannelWaveform.Silent
    }
