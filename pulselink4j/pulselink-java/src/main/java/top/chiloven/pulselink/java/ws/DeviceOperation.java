package top.chiloven.pulselink.java.ws;

import org.jspecify.annotations.Nullable;

/** A V4 {@code device.op} request payload. */
public record DeviceOperation(
        String slotId,
        ActionType action,
        int channel,
        @Nullable Integer priority,
        @Nullable Long durationMs,
        OperateValue value,
        @Nullable Boolean immediate,
        @Nullable Integer ver,
        @Nullable Long seq
) {

    public DeviceOperation {
        if (slotId.isBlank()) {
            throw new IllegalArgumentException("slotId must not be blank");
        }
        if (channel < 0 || channel > 1) {
            throw new IllegalArgumentException("channel must be 0 or 1");
        }
        if (priority != null && (priority < 0 || priority > 2)) {
            throw new IllegalArgumentException("priority must be 0..2");
        }
    }

}
