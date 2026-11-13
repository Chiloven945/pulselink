package top.chiloven.pulselink.device

import top.chiloven.pulselink.core.DeviceKind
import top.chiloven.pulselink.core.PulseLinkException

/** Device session errors. */
public sealed class DeviceError(
    message: String,
    cause: Throwable? = null
) : PulseLinkException(message, cause) {

    public data class Unsupported(
        public val kind: DeviceKind
    ) : DeviceError("unsupported device kind: ${kind.displayName}")

    public data class Initialization(
        public val reason: String,
        override val cause: Throwable? = null
    ) : DeviceError("initialization failed: $reason", cause)

    public data class ProtocolFailure(
        override val cause: Throwable
    ) : DeviceError("protocol failure: ${cause.message}", cause)

    public data class TransportFailure(
        override val cause: Throwable
    ) : DeviceError("transport failure: ${cause.message}", cause)

    public class Closed : DeviceError("session is closed")

}
