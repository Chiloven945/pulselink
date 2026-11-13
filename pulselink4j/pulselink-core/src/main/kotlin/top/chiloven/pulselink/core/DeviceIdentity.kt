package top.chiloven.pulselink.core

/**
 * An opaque BLE device identifier.
 *
 * Platform BLE identifiers are not guaranteed to be stable across restarts or
 * operating systems.
 */
public data class BleDeviceId(public val value: String)

/**
 * A DG-LAB V4 remote device identity.
 *
 * @property clientId the app client that owns the device
 * @property slotId the device slot inside that app
 */
public data class RemoteDeviceId(
    public val clientId: String,
    public val slotId: String,
)

/**
 * A local or remote device identity.
 */
public sealed interface DeviceId {

    /** A BLE device identifier. */
    public data class Ble(public val value: BleDeviceId) : DeviceId

    /** A V4 remote device identifier. */
    public data class Remote(public val value: RemoteDeviceId) : DeviceId

}
