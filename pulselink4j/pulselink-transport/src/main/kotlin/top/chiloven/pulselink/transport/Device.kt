package top.chiloven.pulselink.transport

import java.util.*

/** An opaque platform-specific device identifier. */
@JvmInline
public value class TransportDeviceId(public val value: String)

/** Manufacturer-specific advertisement data. */
public class ManufacturerData(
    public val companyId: Int,
    public val data: ByteArray,
) {

    override fun equals(other: Any?): Boolean =
        other is ManufacturerData
                && companyId == other.companyId
                && data.contentEquals(other.data)

    override fun hashCode(): Int =
        31 * companyId + data.contentHashCode()

    override fun toString(): String =
        "ManufacturerData(companyId=$companyId, bytes=${data.size})"

}

/** A device discovered during a scan. */
public data class DiscoveredDevice(
    public val id: TransportDeviceId,
    public val name: String?,
    public val rssi: Int?,
    public val serviceUuids: List<UUID>,
    public val manufacturerData: List<ManufacturerData> = emptyList(),
    public val connectable: Boolean? = null,
)
