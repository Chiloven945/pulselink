package top.chiloven.pulselink.java.pattern;

/** One 25 ms sample of a waveform timeline. */
public record PulseSample(
        int intervalMillis,
        int intensity
) {

    public PulseSample {
        if (intervalMillis < 10 || intervalMillis > 1000) {
            throw new IllegalArgumentException("intervalMillis must be 10..1000");
        }
        if (intensity < 0 || intensity > 100) {
            throw new IllegalArgumentException("intensity must be 0..100");
        }
    }

}
