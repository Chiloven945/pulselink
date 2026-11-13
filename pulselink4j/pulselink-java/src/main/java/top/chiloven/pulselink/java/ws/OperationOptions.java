package top.chiloven.pulselink.java.ws;

import org.jspecify.annotations.Nullable;

/** Device operation options. */
public record OperationOptions(
        @Nullable Integer priority,
        boolean immediate
) {

    public OperationOptions {
        if (priority != null && (priority < 0 || priority > 2)) {
            throw new IllegalArgumentException("priority must be 0..2");
        }
    }

    public static OperationOptions defaults() {
        return new OperationOptions(null, false);
    }

}
