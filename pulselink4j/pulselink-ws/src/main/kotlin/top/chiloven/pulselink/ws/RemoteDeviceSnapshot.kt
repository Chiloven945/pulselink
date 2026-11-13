package top.chiloven.pulselink.ws

import top.chiloven.pulselink.core.DeviceKind
import kotlinx.serialization.json.JsonObject

/** A cached remote device snapshot. */
public data class RemoteDeviceSnapshot(
    public val id: Int?,
    public val slotId: String,
    public val name: String,
    public val deviceType: String,
    public val props: JsonObject,
    public val slotState: JsonObject,
) {

    /** The device kind derived from [deviceType]. */
    public fun kind(): DeviceKind =
        DeviceKind.fromRemoteType(deviceType)

    /** Typed snapshot for known device families. */
    public fun typed(): TypedRemoteSnapshot =
        when (deviceType) {
            "COYOTE_020", "COYOTE_030" -> TypedRemoteSnapshot.Coyote(this)
            "BMTR_1" -> TypedRemoteSnapshot.Civet(this)
            "OVC_1" -> TypedRemoteSnapshot.Opossum(this)
            else -> TypedRemoteSnapshot.Unknown(this)
        }

}
