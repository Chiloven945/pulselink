package top.chiloven.pulselink.runtime

import top.chiloven.pulselink.core.*
import top.chiloven.pulselink.pattern.Pattern
import top.chiloven.pulselink.runtime.internal.SessionCore
import top.chiloven.pulselink.transport.Backend
import top.chiloven.pulselink.transport.GattLink
import top.chiloven.pulselink.transport.ScanFilter
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow

/** A Coyote V3 session. */
public class CoyoteV3Session internal constructor(
    backend: Backend,
    link: GattLink,
    info: DeviceInfo,
    config: CoyoteV3Config,
    fallbackFilter: ScanFilter,
) : AutoCloseable {

    private val core = SessionCore(backend, link, info, config, fallbackFilter)

    /** The device identity. */
    public val id: DeviceId = DeviceId.Ble(
        BleDeviceId(link.deviceId.value),
    )

    /** Device information. */
    public val deviceInfo: DeviceInfo get() = core.info

    /** Observable session state. */
    public val state: StateFlow<CoyoteV3State> get() = core.state

    /** Session events. */
    public val events: SharedFlow<CoyoteV3Event> get() = core.events

    internal suspend fun initialize() {
        core.initialize()
    }

    /** Sets an absolute intensity. */
    public suspend fun setIntensity(channel: Channel, intensity: Intensity): IntensityChange =
        core.intensityController.setIntensity(channel, intensity)

    /** Adjusts an intensity by a relative delta. */
    public suspend fun adjustIntensity(channel: Channel, delta: IntensityDelta): IntensityChange =
        core.intensityController.adjustIntensity(channel, delta)

    /** Updates the per-channel soft limit. */
    public suspend fun setSoftLimit(channel: Channel, intensity: Intensity) {
        core.intensityController.setSoftLimit(channel, intensity)
    }

    /** Updates balance parameters. */
    public suspend fun setBalance(balance: CoyoteV3Balance) {
        core.intensityController.setBalance(balance)
    }

    /** Starts pattern playback. */
    public suspend fun play(
        pattern: Pattern,
        options: PlayOptions = PlayOptions()
    ): PlayHandle =
        core.playbackController.play(pattern, options)

    /** Stops playback and sends zero intensity. */
    public suspend fun stop(): StopOutcome =
        core.playbackController.stop()

    /** Reconnects and reinitializes the session. */
    public suspend fun reconnect() {
        core.reconnectController.reconnect()
    }

    override fun close() {
        core.close()
    }

}
