package top.chiloven.pulselink.transport

import java.util.*

/** A device discovered during a scan. */
public data class DiscoveredDevice(
    public val id: TransportDeviceId,
    public val name: String?,
    public val rssi: Int?,
    public val serviceUuids: List<UUID>,
    public val manufacturerData: List<ManufacturerData> = emptyList(),
    public val connectable: Boolean? = null,
)
