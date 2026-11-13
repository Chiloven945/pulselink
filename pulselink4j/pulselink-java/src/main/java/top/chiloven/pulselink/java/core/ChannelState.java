package top.chiloven.pulselink.java.core;

import org.jspecify.annotations.Nullable;

/** Observed state for one channel. */
public record ChannelState(
        @Nullable Integer intensity,
        @Nullable Integer softLimit
) {

}
