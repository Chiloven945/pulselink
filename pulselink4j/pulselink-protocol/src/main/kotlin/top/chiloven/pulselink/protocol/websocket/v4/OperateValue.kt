package top.chiloven.pulselink.protocol.websocket.v4

/** The `v` field of a device operation. */
public sealed interface OperateValue {

    public data class Intensity(public val value: Int) : OperateValue

    public data class PulseFrames(public val frames: List<PulseFrameWire>) : OperateValue

}
