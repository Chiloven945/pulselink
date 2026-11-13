package top.chiloven.pulselink.runtime

import top.chiloven.pulselink.core.ChannelMap

/** Coyote V3 balance parameters. */
public data class CoyoteV3Balance(
    public val frequency: ChannelMap<Int> = ChannelMap(128, 128),
    public val intensity: ChannelMap<Int> = ChannelMap(128, 128),
)
