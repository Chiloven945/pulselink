package top.chiloven.pulselink.ws

import top.chiloven.pulselink.core.Channel as CoreChannel

/** The scope of a `device.op.clear`. */
public sealed interface ClearTarget {

    public data object All : ClearTarget

    public data class Device(
        public val slotId: String
    ) : ClearTarget

    public data class Channel(
        public val slotId: String,
        public val channel: CoreChannel,
    ) : ClearTarget

}
