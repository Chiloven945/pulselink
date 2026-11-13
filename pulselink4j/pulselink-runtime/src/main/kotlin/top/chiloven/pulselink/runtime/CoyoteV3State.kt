package top.chiloven.pulselink.runtime

import top.chiloven.pulselink.core.DeviceState
import top.chiloven.pulselink.core.Observed
import top.chiloven.pulselink.protocol.coyote.v3.B1Message

/** The observable state of a Coyote V3 session. */
public data class CoyoteV3State(
    public val common: DeviceState,
    public val settings: Observed<CoyoteV3Settings> = Observed.Unknown,
    public val pattern: PatternPlaybackState? = null,
    public val lastB1: B1Message? = null,
)
