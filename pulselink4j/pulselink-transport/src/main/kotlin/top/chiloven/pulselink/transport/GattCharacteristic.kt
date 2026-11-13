package top.chiloven.pulselink.transport

import java.util.*

/** A GATT characteristic identified by its service and characteristic UUIDs. */
public data class GattCharacteristic(
    public val service: UUID,
    public val characteristic: UUID,
)
