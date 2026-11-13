package top.chiloven.pulselink.protocol.websocket.v4

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/** A V4 `device.op` request payload. */
@Serializable
public data class DeviceOperate(
    @SerialName("s") public val slotId: String,
    @SerialName("t") @Serializable(with = ActionTypeSerializer::class) public val action: ActionType,
    @SerialName("c") public val channel: Int,
    @SerialName("p") public val priority: Int? = null,
    @SerialName("d") public val durationMs: Long? = null,
    @SerialName("v") @Serializable(with = OperateValueSerializer::class) public val value: OperateValue,
    @SerialName("im") public val immediate: Boolean? = null,
    @SerialName("ver") public val ver: Int? = null,
    @SerialName("seq") public val seq: Long? = null,
) {

    /** Validates cross-field constraints that serde cannot express. */
    public fun validate(): DeviceOperate =
        validateDeviceOperate(this)

}
