package top.chiloven.pulselink.runtime

import top.chiloven.pulselink.core.Channel
import top.chiloven.pulselink.core.Intensity

/** The result of an intensity change. */
public data class IntensityChange(
    public val channel: Channel,
    public val requested: Intensity,
    public val applied: Intensity,
)
