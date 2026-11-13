package top.chiloven.pulselink.core

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
