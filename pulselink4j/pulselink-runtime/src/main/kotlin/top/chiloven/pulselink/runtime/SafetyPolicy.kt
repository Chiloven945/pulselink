package top.chiloven.pulselink.runtime

import top.chiloven.pulselink.core.ChannelMap
import top.chiloven.pulselink.core.Intensity
import top.chiloven.pulselink.core.IntensityDelta

/** Safety limits applied before intensity operations. */
public data class SafetyPolicy(
    public val maxIntensity: ChannelMap<Intensity> = ChannelMap(Intensity.MAX, Intensity.MAX),
    public val maxDelta: ChannelMap<IntensityDelta>? = null,
    public val allowTemporary: Boolean = true,
    public val requireSoftLimit: Boolean = false,
)
