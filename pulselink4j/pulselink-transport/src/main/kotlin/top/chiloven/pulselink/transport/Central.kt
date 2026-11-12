package top.chiloven.pulselink.transport

import kotlin.time.Duration

/** A source of devices and links. */
public interface Central {

    /**
     * Scans for devices for up to [timeout].
     *
     * Returns devices discovered so far when the timeout expires. Cancelling
     * the call must stop the underlying scan.
     */
    public suspend fun scan(
        filter: ScanFilter = ScanFilter.all(),
        timeout: Duration,
    ): List<DiscoveredDevice>

    /** Connects to a discovered device. */
    public suspend fun connect(device: DiscoveredDevice): GattLink

}

/** A target used for reconnection. */
public data class ReconnectTarget(
    public val deviceId: TransportDeviceId,
    public val filter: ScanFilter = ScanFilter.all(),
)

/** Re-establishes a previously known link. */
public interface Connector {

    public suspend fun reconnect(target: ReconnectTarget): GattLink

}

/** A complete local-device backend. */
public interface Backend : Central, Connector
