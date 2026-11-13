package top.chiloven.pulselink.java.runtime;

import java.time.Duration;

/** Current pattern playback snapshot. */
public record PatternPlaybackState(
        long id,
        Duration elapsed,
        int frameIndex,
        boolean looped
) {

}
