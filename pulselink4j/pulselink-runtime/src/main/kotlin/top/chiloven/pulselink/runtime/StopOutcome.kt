package top.chiloven.pulselink.runtime

import top.chiloven.pulselink.core.ChannelMap
import top.chiloven.pulselink.core.Intensity
import top.chiloven.pulselink.core.Observed

/** The result of stopping output. */
public data class StopOutcome(
    public val acknowledged: Boolean,
    public val intensity: ChannelMap<Observed<Intensity>>,
)
