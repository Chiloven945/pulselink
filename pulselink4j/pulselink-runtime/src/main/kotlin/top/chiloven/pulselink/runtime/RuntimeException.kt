package top.chiloven.pulselink.runtime

import top.chiloven.pulselink.core.DeviceKind
import top.chiloven.pulselink.core.PulseLinkException

/** Runtime errors. */
public sealed class RuntimeException(
    message: String,
    cause: Throwable? = null
) : PulseLinkException(message, cause) {

    public class NotReady : RuntimeException("session is not ready")

    public data class Unsupported(
        public val kind: DeviceKind
    ) : RuntimeException("unsupported device kind: ${kind.displayName}")

    public data class AckTimeout(
        public val sequence: Int
    ) : RuntimeException("B0 acknowledgement timed out for sequence $sequence")

    public class Cancelled : RuntimeException("operation cancelled")

    public class Closed : RuntimeException("session is closed")

    public data class SafetyLimit(
        public val reason: String
    ) : RuntimeException("safety limit exceeded: $reason")

    public class Busy : RuntimeException("session command queue is full")

    public data class ReconnectFailed(
        public val attempts: Int
    ) : RuntimeException("reconnect failed after $attempts attempts")

}
