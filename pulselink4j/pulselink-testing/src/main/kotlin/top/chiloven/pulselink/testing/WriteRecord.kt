package top.chiloven.pulselink.testing

import top.chiloven.pulselink.transport.GattCharacteristic
import top.chiloven.pulselink.transport.WriteMode

/** A recorded write. */
public class WriteRecord(
    public val characteristic: GattCharacteristic,
    public val data: ByteArray,
    public val mode: WriteMode,
) {

    override fun equals(other: Any?): Boolean =
        other is WriteRecord &&
                characteristic == other.characteristic &&
                data.contentEquals(other.data) &&
                mode == other.mode

    override fun hashCode(): Int =
        31 * (31 * characteristic.hashCode() + data.contentHashCode()) + mode.hashCode()

    override fun toString(): String =
        "WriteRecord(characteristic=$characteristic, bytes=${data.size}, mode=$mode)"

}
