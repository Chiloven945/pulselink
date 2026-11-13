package top.chiloven.pulselink.transport

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
