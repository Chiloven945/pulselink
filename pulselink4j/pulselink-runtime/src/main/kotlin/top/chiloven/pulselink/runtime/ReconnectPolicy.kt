package top.chiloven.pulselink.runtime

import kotlin.time.Duration
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.Duration.Companion.seconds

/** Reconnect policy. */
public data class ReconnectPolicy(
    public val enabled: Boolean = true,
    public val maxAttempts: Int = 5,
    public val initialBackoff: Duration = 500.milliseconds,
    public val maxBackoff: Duration = 30.seconds,
    public val jitter: Boolean = true,
)
