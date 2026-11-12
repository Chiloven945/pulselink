package top.chiloven.pulselink.pattern

import top.chiloven.pulselink.core.PulseLinkException

/** Errors returned by waveform timeline types. */
public sealed class PatternException(message: String) : PulseLinkException(message) {

    public data class InvalidInterval(
        public val milliseconds: Int
    ) : PatternException("invalid pulse interval $milliseconds; expected 10..1000")

    public data class InvalidFrequency(
        public val hz: Double
    ) : PatternException("invalid pulse frequency $hz")

    public data class InvalidPulseIntensity(
        public val value: Int
    ) : PatternException("invalid pulse intensity $value; expected 0..100")

    public data class InvalidSampleCount(
        public val count: Int
    ) : PatternException("active channel frame requires 4 samples, got $count")

    public class EmptyTimeline : PatternException("timeline must not be empty")

}
