package top.chiloven.pulselink.pattern

/** A frame cursor over a [Pattern]. */
public class PatternCursor(private val pattern: Pattern) {

    private var index: Int = 0
    private var finished: Boolean = false

    /** The current frame index. */
    public val frameIndex: Int get() = index

    /** Returns whether the cursor has passed the end of a non-looping pattern. */
    public val isFinished: Boolean get() = finished

    /** Returns the current frame. */
    public fun currentFrame(): StereoFrame =
        pattern.timeline.frame(index)
            ?: throw PatternException.EmptyTimeline()

    /**
     * Advances to the next frame.
     *
     * @return true when a new frame is available, false at the end of a
     * non-looping pattern.
     */
    public fun advance(): Boolean {
        if (finished) return false
        val next = index + 1
        if (next < pattern.timeline.size) {
            index = next
            return true
        }
        if (pattern.looped) {
            index = 0
            return true
        }
        finished = true
        return false
    }

}
