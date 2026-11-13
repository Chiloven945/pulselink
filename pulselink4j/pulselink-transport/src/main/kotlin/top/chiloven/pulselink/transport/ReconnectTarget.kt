package top.chiloven.pulselink.transport

/** A target used for reconnection. */
public data class ReconnectTarget(
    public val deviceId: TransportDeviceId,
    public val filter: ScanFilter = ScanFilter.all(),
)
