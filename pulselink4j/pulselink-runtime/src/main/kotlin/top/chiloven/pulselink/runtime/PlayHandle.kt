package top.chiloven.pulselink.runtime

import kotlinx.coroutines.CompletableDeferred

/** A handle for one pattern playback. */
public class PlayHandle internal constructor(public val id: PatternId) {

    private val completion = CompletableDeferred<PlayEndReason>()

    /** Whether playback has finished. */
    public val isFinished: Boolean get() = completion.isCompleted

    /** Waits for playback to end. */
    public suspend fun wait(): PlayEndReason =
        completion.await()

    /** Stops playback and returns its end reason. */
    public suspend fun stop(): PlayEndReason {
        finish(PlayEndReason.Stopped)
        return completion.await()
    }

    internal fun finish(reason: PlayEndReason) {
        completion.complete(reason)
    }

}
