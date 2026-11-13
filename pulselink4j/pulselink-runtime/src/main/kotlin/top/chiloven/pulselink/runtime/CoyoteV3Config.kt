package top.chiloven.pulselink.runtime

import top.chiloven.pulselink.core.ChannelMap
import top.chiloven.pulselink.core.Intensity
import kotlin.time.Duration
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.Duration.Companion.seconds

/** Coyote V3 session configuration. */
public data class CoyoteV3Config(
    public val ackTimeout: Duration = 1.seconds,
    public val tickInterval: Duration = 100.milliseconds,
    public val lagThreshold: Duration = 500.milliseconds,
    public val safety: SafetyPolicy = SafetyPolicy(),
    public val reconnect: ReconnectPolicy = ReconnectPolicy(),
    public val initialSoftLimit: ChannelMap<Intensity> = ChannelMap(Intensity.MAX, Intensity.MAX),
    public val balance: CoyoteV3Balance? = null,
)
