package top.chiloven.pulselink.ws

import top.chiloven.pulselink.core.Channel
import top.chiloven.pulselink.core.DeviceKind
import top.chiloven.pulselink.core.Intensity
import top.chiloven.pulselink.protocol.websocket.v4.ActionType
import top.chiloven.pulselink.protocol.websocket.v4.DeviceOperate
import top.chiloven.pulselink.protocol.websocket.v4.OperateValue
import top.chiloven.pulselink.protocol.websocket.v4.PulseFrameWire
import kotlin.time.Duration

/** A handle for one remote device. */
public class RemoteDeviceHandle internal constructor(
    private val app: AppClient,
    public val slotId: String,
) {

    /** Current snapshot, if still cached. */
    public fun snapshot(): RemoteDeviceSnapshot? =
        app.devices().firstOrNull { it.slotId == slotId }

    /** Device kind. */
    public fun kind(): DeviceKind =
        snapshot()?.kind() ?: DeviceKind.Unknown("unknown")

    /** Typed snapshot. */
    public fun typed(): TypedRemoteSnapshot? =
        snapshot()?.typed()

    /** Resets intensity to zero. */
    public suspend fun resetIntensity(
        channel: Channel,
        options: OperationOptions = OperationOptions(),
    ): OperationHandle =
        intensity(
            ActionType.SetIntensity,
            channel,
            0,
            null,
            options
        )

    /** Adds intensity. */
    public suspend fun addIntensity(
        channel: Channel,
        value: Int,
        options: OperationOptions = OperationOptions(),
    ): OperationHandle =
        intensity(
            ActionType.AddIntensity,
            channel,
            value,
            null,
            options
        )

    /** Reduces intensity. */
    public suspend fun reduceIntensity(
        channel: Channel,
        value: Int,
        options: OperationOptions = OperationOptions(),
    ): OperationHandle =
        intensity(
            ActionType.AddIntensity,
            channel,
            -value,
            null,
            options
        )

    /** Applies a temporary intensity. */
    public suspend fun setTempIntensity(
        channel: Channel,
        intensity: Intensity,
        duration: Duration,
        options: OperationOptions = OperationOptions(),
    ): OperationHandle =
        intensity(
            ActionType.SetTempIntensity,
            channel,
            intensity.value,
            duration.inWholeMilliseconds,
            options,
        )

    /** Sends v3 pulse frames. */
    public suspend fun sendPulse(
        channel: Channel,
        duration: Duration,
        frames: List<ByteArray>,
        options: OperationOptions = OperationOptions(),
    ): OperationHandle =
        pulse(
            channel,
            duration,
            frames,
            3,
            options
        )

    /** Sends v2 pulse frames. */
    public suspend fun sendPulseV2(
        channel: Channel,
        duration: Duration,
        frames: List<ByteArray>,
        options: OperationOptions = OperationOptions(),
    ): OperationHandle =
        pulse(
            channel,
            duration,
            frames,
            2,
            options
        )

    private suspend fun intensity(
        action: ActionType,
        channel: Channel,
        value: Int,
        durationMs: Long?,
        options: OperationOptions,
    ): OperationHandle =
        app.submitOperate(
            DeviceOperate(
                slotId = slotId,
                action = action,
                channel = channel.wireValue,
                priority = options.priority,
                durationMs = durationMs,
                value = OperateValue.Intensity(value),
                immediate = options.immediate,
            ),
            options,
        )

    private suspend fun pulse(
        channel: Channel,
        duration: Duration,
        frames: List<ByteArray>,
        version: Int,
        options: OperationOptions,
    ): OperationHandle =
        app.submitOperate(
            DeviceOperate(
                slotId = slotId,
                action = ActionType.AppendPulseData,
                channel = channel.wireValue,
                priority = options.priority,
                durationMs = duration.inWholeMilliseconds,
                value = OperateValue.PulseFrames(
                    frames.map {
                        PulseFrameWire.Hex(
                            it.joinToString("") { byte ->
                                "%02X".format(byte)
                            }
                        )
                    },
                ),
                immediate = options.immediate,
                ver = version,
            ),
            options,
        )

}
