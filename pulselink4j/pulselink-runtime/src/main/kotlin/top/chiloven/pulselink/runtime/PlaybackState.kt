package top.chiloven.pulselink.runtime

/** Pattern playback state. */
public sealed interface PlaybackState {

    public data object Idle : PlaybackState

    public data class Playing(
        public val id: PatternId,
        public val frameIndex: Int
    ) : PlaybackState

    public data class Finished(
        public val id: PatternId,
        public val reason: PlayEndReason
    ) : PlaybackState

}
