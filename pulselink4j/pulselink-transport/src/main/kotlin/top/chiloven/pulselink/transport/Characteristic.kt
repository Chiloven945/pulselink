package top.chiloven.pulselink.transport

import java.util.*

/** A GATT characteristic identified by its service and characteristic UUIDs. */
public data class GattCharacteristic(
    public val service: UUID,
    public val characteristic: UUID,
)

/** Capabilities of a discovered GATT characteristic. */
public data class CharacteristicInfo(
    public val id: GattCharacteristic,
    public val readable: Boolean,
    public val writable: Boolean,
    public val writableWithoutResponse: Boolean,
    public val notifiable: Boolean,
    public val indicatable: Boolean,
)

/** The write mode used for a GATT characteristic. */
public enum class WriteMode {

    /** Choose based on characteristic properties. */
    Auto,

    /** Require a write response. */
    WithResponse,

    /** Use a write command without a response. */
    WithoutResponse,

}
