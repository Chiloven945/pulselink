package top.chiloven.pulselink.protocol.websocket.v4

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/** The result of a completed `device.op` request. */
@Serializable
public data class DeviceOperateResult(
    @SerialName("type") @Serializable(with = ActionTypeSerializer::class) public val action: ActionType,
    public val reason: OperationEndReason,
    @SerialName("slotId") public val slotId: String? = null,
    public val channel: Int? = null,
)
