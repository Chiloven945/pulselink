package top.chiloven.pulselink.runtime

import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds

/** Client-level configuration. */
public data class ClientConfig(
    public val commandCapacity: Int = 64,
    public val eventCapacity: Int = 256,
    public val connectTimeout: Duration = 15.seconds,
)
