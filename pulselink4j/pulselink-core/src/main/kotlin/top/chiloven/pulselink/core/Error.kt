package top.chiloven.pulselink.core

/** Base class for all public pulselink exceptions. */
public abstract class PulseLinkException(
    message: String,
    cause: Throwable? = null,
) : Exception(message, cause)

/** Errors returned when constructing or converting core domain values. */
public sealed class CoreException(message: String) : PulseLinkException(message) {

    public data class InvalidIntensity(
        public val value: Int
    ) : CoreException("invalid intensity $value; expected 0..200")

    public data class InvalidIntensityDelta(
        public val value: Int
    ) : CoreException("invalid intensity delta $value; expected -200..200")

    public data class InvalidBatteryLevel(
        public val value: Int
    ) : CoreException("invalid battery level $value; expected 0..100")

    public data class InvalidChannel(
        public val value: Int
    ) : CoreException("invalid channel $value; expected 0 (A) or 1 (B)")

    public data class InvalidButtonMask(
        public val value: Int
    ) : CoreException("invalid button mask $value; expected 0..65535")

    public data class InvalidTrigger(
        public val eventId: Int,
        public val parameter: Int
    ) : CoreException("invalid trigger event=$eventId parameter=$parameter; expected 0..255")

}
