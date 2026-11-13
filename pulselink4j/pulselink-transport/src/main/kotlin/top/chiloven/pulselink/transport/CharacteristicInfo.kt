package top.chiloven.pulselink.transport

/** Capabilities of a discovered GATT characteristic. */
public data class CharacteristicInfo(
    public val id: GattCharacteristic,
    public val readable: Boolean,
    public val writable: Boolean,
    public val writableWithoutResponse: Boolean,
    public val notifiable: Boolean,
    public val indicatable: Boolean,
)
