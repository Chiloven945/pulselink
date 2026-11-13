package top.chiloven.pulselink.runtime

import kotlin.time.Duration

/** Current pattern playback snapshot. */
public data class PatternPlaybackState(
    public val id: PatternId,
    public val elapsed: Duration,
    public val frameIndex: Int,
    public val looped: Boolean,
)
