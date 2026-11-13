package top.chiloven.pulselink.ws

import top.chiloven.pulselink.core.Channel
import top.chiloven.pulselink.protocol.websocket.v4.ActionType
import top.chiloven.pulselink.protocol.websocket.v4.OperationEndReason

/** The result of a completed operation. */
public data class OperationResult(
    public val action: ActionType,
    public val reason: OperationEndReason,
    public val slotId: String?,
    public val channel: Channel?,
)
