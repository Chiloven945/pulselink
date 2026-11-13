package top.chiloven.pulselink.pattern

/** A playable, optionally looping timeline. */
public class Pattern private constructor(
    public val timeline: Timeline,
    public val looped: Boolean,
) {

    public companion object {

        /** Creates a pattern from a non-empty timeline. */
        public fun of(
            timeline: Timeline,
            looped: Boolean = false
        ): Pattern =
            Pattern(timeline, looped)

    }

}
