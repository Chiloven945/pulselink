package top.chiloven.pulselink.core

/**
 * The lifecycle state of a device session.
 */
public enum class ConnectionState {

    Disconnected,
    Connecting,
    Initializing,
    Ready,
    Degraded,
    Reconnecting,
    Closing,

}

/**
 * A value that may not have been observed yet.
 *
 * `Unknown` is not the same as `Known(0)`.
 */
public sealed interface Observed<out T> {

    public data object Unknown : Observed<Nothing>
    public data class Known<T>(public val value: T) : Observed<T>

    /** Returns the known value, if any. */
    public fun valueOrNull(): T? =
        when (this) {
            Unknown -> null
            is Known -> value
        }

    public companion object {

        /** Creates an unknown value. */
        public fun <T> unknown(): Observed<T> =
            Unknown

        /** Creates a known value. */
        public fun <T> known(value: T): Observed<T> =
            Known(value)

    }

}

/** Transforms a known value. */
public fun <T, R> Observed<T>.mapObserved(transform: (T) -> R): Observed<R> =
    when (this) {
        Observed.Unknown -> Observed.Unknown
        is Observed.Known -> Observed.Known(transform(value))
    }

/**
 * A battery percentage in the range `0..=100`.
 */
@JvmInline
public value class BatteryLevel private constructor(
    public val value: Int
) : Comparable<BatteryLevel> {

    override fun compareTo(other: BatteryLevel): Int =
        value.compareTo(other.value)

    override fun toString(): String =
        "$value%"

    public companion object {

        public val MIN: BatteryLevel = BatteryLevel(0)
        public val MAX: BatteryLevel = BatteryLevel(100)

        public fun of(value: Int): BatteryLevel =
            when (value) {
                in 0..100 -> BatteryLevel(value)
                else -> throw CoreException.InvalidBatteryLevel(value)
            }

    }

}

/** Observed battery information. */
public data class BatteryState(public val level: BatteryLevel)

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
