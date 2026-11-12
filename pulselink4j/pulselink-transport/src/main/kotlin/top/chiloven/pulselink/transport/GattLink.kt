package top.chiloven.pulselink.transport

import kotlinx.coroutines.flow.Flow

/** A bidirectional local GATT link. */
public interface GattLink {

    /** The device id for this link. */
    public val deviceId: TransportDeviceId

    /** Whether the link is currently believed to be connected. */
    public val isConnected: Boolean

    /** The negotiated ATT MTU, including the ATT header, or null when unknown. */
    public fun mtu(): Int?

    /** Returns the discovered characteristics keyed by service/characteristic UUID. */
    public fun characteristics(): Map<GattCharacteristic, CharacteristicInfo>

    /** Returns one discovered characteristic, if present. */
    public fun characteristic(id: GattCharacteristic): CharacteristicInfo?

    /** Writes bytes to a characteristic. */
    public suspend fun write(
        characteristic: GattCharacteristic,
        data: ByteArray,
        mode: WriteMode = WriteMode.Auto,
    )

    /** Reads bytes from a characteristic. */
    public suspend fun read(characteristic: GattCharacteristic): ByteArray

    /**
     * Returns a cold notification stream for [characteristic].
     *
     * Each collector establishes its own subscription; cancelling the
     * collector cancels that subscription.
     */
    public fun subscribe(characteristic: GattCharacteristic): Flow<ByteArray>

    /** Disconnects the link. Implementations must make this idempotent. */
    public suspend fun disconnect()

}
