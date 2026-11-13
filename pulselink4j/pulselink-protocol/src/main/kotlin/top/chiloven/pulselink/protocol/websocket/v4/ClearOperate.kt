package top.chiloven.pulselink.protocol.websocket.v4

import top.chiloven.pulselink.protocol.ProtocolException
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/** A V4 `device.op.clear` request payload. */
@Serializable
public data class ClearOperate(
    @SerialName("s") public val slotId: String? = null,
    @SerialName("c") public val channel: Int? = null,
) {

    /** Validates that a channel is not specified without a slot id. */
    public fun validate(): ClearOperate {
        channel?.let {
            when {
                slotId == null -> throw ProtocolException.InvalidParameters(
                    "device.op.clear channel requires slotId"
                )

                it !in 0..1 -> throw ProtocolException.ValueOutOfRange(
                    "clear channel",
                    it.toLong(),
                    0,
                    1
                )
            }
        }
        return this
    }

}
