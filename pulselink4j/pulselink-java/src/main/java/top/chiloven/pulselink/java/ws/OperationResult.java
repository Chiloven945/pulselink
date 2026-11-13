package top.chiloven.pulselink.java.ws;

import top.chiloven.pulselink.java.core.Channel;

import org.jspecify.annotations.Nullable;

/** The result of a completed operation. */
public record OperationResult(
        ActionType action,
        OperationEndReason reason,
        @Nullable String slotId,
        @Nullable Channel channel
) {

}
