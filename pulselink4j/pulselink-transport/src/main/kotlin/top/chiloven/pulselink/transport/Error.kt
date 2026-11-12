package top.chiloven.pulselink.transport

import top.chiloven.pulselink.core.PulseLinkException

/** Errors returned by local device transports. */
public sealed class TransportException(
    message: String,
    cause: Throwable? = null,
) : PulseLinkException(message, cause) {

    public class NoAdapter : TransportException("no BLE adapter available")

    public data class ScanFailed(
        public val reason: String,
        override val cause: Throwable? = null
    ) : TransportException("scan failed: $reason", cause)

    public data class DeviceNotFound(
        public val id: TransportDeviceId
    ) : TransportException("device not found: ${id.value}")

    public data class ConnectFailed(
        public val reason: String,
        override val cause: Throwable? = null
    ) : TransportException("connection failed: $reason", cause)

    public data class GattFailed(
        public val reason: String,
        override val cause: Throwable? = null
    ) : TransportException("GATT operation failed: $reason", cause)

    public data class NotifyFailed(
        public val reason: String,
        override val cause: Throwable? = null
    ) : TransportException("notification operation failed: $reason", cause)

    public class Disconnected : TransportException("link disconnected")

    public class Timeout : TransportException("operation timed out")

    public data class Backend(
        public val reason: String,
        override val cause: Throwable? = null
    ) : TransportException("transport backend error: $reason", cause)

}
