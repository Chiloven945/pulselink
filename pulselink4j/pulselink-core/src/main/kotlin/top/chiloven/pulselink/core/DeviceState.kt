package top.chiloven.pulselink.core

/** Observed state for one channel. */
public data class ChannelState(
    public val intensity: Observed<Intensity> = Observed.Unknown,
    public val softLimit: Observed<Intensity> = Observed.Unknown,
)

/**
 * An immutable snapshot of a device's observed state.
 */
public data class DeviceState(
    public val revision: Long,
    public val connection: ConnectionState,
    public val battery: Observed<BatteryState> = Observed.Unknown,
    public val channels: ChannelMap<ChannelState> = ChannelMap(ChannelState(), ChannelState()),
) {

    /** Returns the observed state for one channel. */
    public fun channel(channel: Channel): ChannelState =
        channels[channel]

    /** Returns a copy marked as updated. */
    public fun touch(): DeviceState =
        copy(revision = revision.saturatingAddOne())

    private fun Long.saturatingAddOne(): Long =
        when (Long.MAX_VALUE) {
            this -> this
            else -> this + 1
        }

    public companion object {

        /** Creates an empty state snapshot. */
        public fun empty(connection: ConnectionState): DeviceState =
            DeviceState(
                revision = 0,
                connection = connection,
            )

    }

}
