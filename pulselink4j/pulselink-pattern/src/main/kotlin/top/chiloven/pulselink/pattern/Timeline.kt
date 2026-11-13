package top.chiloven.pulselink.pattern

import kotlin.time.Duration

/** The duration of one frame in milliseconds. */
public const val FRAME_DURATION_MS: Long = 100

/** A non-empty sequence of 100 ms stereo frames. */
public class Timeline private constructor(
    private val frames: List<StereoFrame>,
) {

    /** Returns the number of frames. */
    public val size: Int get() = frames.size

    /** Returns whether this timeline is empty. A valid timeline is never empty. */
    public fun isEmpty(): Boolean = frames.isEmpty()

    /** Returns the frames. */
    public fun frames(): List<StereoFrame> = frames

    /** Returns the frame at [index], if present. */
    public fun frame(index: Int): StereoFrame? = frames.getOrNull(index)

    /** Returns the frame at a 100 ms frame index derived from [elapsed]. */
    public fun frameAtElapsed(elapsed: Duration): StereoFrame? {
        val millis = elapsed.inWholeMilliseconds
        if (millis < 0) return null
        val index = millis / FRAME_DURATION_MS
        return frames.getOrNull(index.toInt())
    }

    override fun equals(other: Any?): Boolean = other is Timeline && frames == other.frames

    override fun hashCode(): Int = frames.hashCode()

    override fun toString(): String = "Timeline(frames=$frames)"

    public companion object {

        /** Creates a non-empty timeline. */
        public fun of(frames: List<StereoFrame>): Timeline {
            if (frames.isEmpty()) throw PatternException.EmptyTimeline()
            return Timeline(frames.toList())
        }
    }
}
