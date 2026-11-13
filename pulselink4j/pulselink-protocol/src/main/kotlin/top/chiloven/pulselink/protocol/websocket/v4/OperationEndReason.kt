package top.chiloven.pulselink.protocol.websocket.v4

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/** Why a device operation ended. */
@Serializable
public enum class OperationEndReason {

    @SerialName("completed")
    Completed,

    @SerialName("cleared")
    Cleared,

    @SerialName("replaced")
    Replaced,

    @SerialName("cancelled")
    Cancelled,

}
