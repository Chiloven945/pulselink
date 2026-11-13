package top.chiloven.pulselink.java.runtime;

import top.chiloven.pulselink.java.core.DeviceState;

import org.jspecify.annotations.Nullable;

/** The observable state of a Coyote V3 session. */
public record CoyoteV3State(
        DeviceState common,
        @Nullable CoyoteV3Settings settings,
        @Nullable PatternPlaybackState pattern,
        @Nullable B1Snapshot lastB1
) {

}
