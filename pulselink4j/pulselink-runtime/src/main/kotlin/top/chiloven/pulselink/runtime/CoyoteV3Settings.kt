package top.chiloven.pulselink.runtime

import top.chiloven.pulselink.core.ChannelMap
import top.chiloven.pulselink.core.Intensity

/** Current V3 settings. */
public data class CoyoteV3Settings(
    public val softLimit: ChannelMap<Intensity>,
    public val frequencyBalance: ChannelMap<Int>,
    public val intensityBalance: ChannelMap<Int>,
)
