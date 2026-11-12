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

/**
 * The kind of DG-LAB device.
 */
public sealed interface DeviceKind {

    public data object CoyoteV2 : DeviceKind
    public data object CoyoteV3 : DeviceKind
    public data object PawPrints : DeviceKind
    public data object Civet : DeviceKind
    public data object Opossum : DeviceKind
    public data class Unknown(public val value: String) : DeviceKind

    /** The DG-LAB V4 device type, when one exists. */
    public val remoteType: String?
        get() = when (this) {
            CoyoteV2 -> "COYOTE_020"
            CoyoteV3 -> "COYOTE_030"
            Civet -> "BMTR_1"
            Opossum -> "OVC_1"
            PawPrints, is Unknown -> null
        }

    /** A human-readable name. */
    public val displayName: String
        get() = when (this) {
            CoyoteV2 -> "Coyote V2"
            CoyoteV3 -> "Coyote V3"
            PawPrints -> "PawPrints"
            Civet -> "Civet"
            Opossum -> "Opossum"
            is Unknown -> value
        }

    public companion object {

        /** Maps a V4 device type to a kind. */
        public fun fromRemoteType(value: String): DeviceKind =
            when (value) {
                "COYOTE_020" -> CoyoteV2
                "COYOTE_030" -> CoyoteV3
                "BMTR_1" -> Civet
                "OVC_1" -> Opossum
                else -> Unknown(value)
            }

    }

}

/**
 * Descriptive information about a device.
 */
public data class DeviceInfo(
    public val id: DeviceId,
    public val kind: DeviceKind,
    public val name: String,
    public val manufacturer: String? = null,
    public val model: String? = null,
    public val firmware: String? = null,
)
