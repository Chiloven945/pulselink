package top.chiloven.pulselink.ws

import kotlin.time.Duration

/** Device operation options. */
public data class OperationOptions(
    public val priority: Int? = null,
    public val immediate: Boolean = false,
    public val timeout: Duration? = null,
)
