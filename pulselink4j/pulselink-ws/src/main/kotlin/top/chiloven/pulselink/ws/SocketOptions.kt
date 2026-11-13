package top.chiloven.pulselink.ws

import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds

/** WebSocket connection options. */
public data class SocketOptions(
    public val url: String,
    public val connectTimeout: Duration = 10.seconds,
    public val responseTimeout: Duration = 8.seconds,
    public val pingInterval: Duration? = null,
    public val maxFrameSize: Long = 8L * 1024 * 1024,
)
