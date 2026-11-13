package top.chiloven.pulselink.java.core;

import org.jspecify.annotations.Nullable;

/** Descriptive information about a device. */
public record DeviceInfo(
        DeviceId id,
        DeviceKind kind,
        String name,
        @Nullable String manufacturer,
        @Nullable String model,
        @Nullable String firmware
) {

}
