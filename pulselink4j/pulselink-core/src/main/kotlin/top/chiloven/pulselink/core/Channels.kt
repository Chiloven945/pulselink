package top.chiloven.pulselink.core

/**
 * A device output channel.
 */
public enum class Channel(
    public val index: Int,
    public val wireValue: Int
) {

    A(0, 0),
    B(1, 1);

    public companion object {

        /** The two channels in wire order. */
        public val ALL: List<Channel> = listOf(A, B)

        /** Decodes the V4 channel discriminator. */
        public fun fromWire(value: Int): Channel =
            when (value) {
                0 -> A
                1 -> B
                else -> throw CoreException.InvalidChannel(value)
            }

    }

}

/**
 * A value for each output channel.
 */
public data class ChannelMap<T>(
    public val a: T,
    public val b: T,
) {

    /** Returns the value for [channel]. */
    public operator fun get(channel: Channel): T =
        when (channel) {
            Channel.A -> a
            Channel.B -> b
        }

    /** Transforms both values. */
    public fun <R> map(transform: (T) -> R): ChannelMap<R> =
        ChannelMap(
            a = transform(a),
            b = transform(b),
        )

    /** Returns both channel/value pairs. */
    public fun entries(): List<Pair<Channel, T>> =
        listOf(Channel.A to a, Channel.B to b)

}
