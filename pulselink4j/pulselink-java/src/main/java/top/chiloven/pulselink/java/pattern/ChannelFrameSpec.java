package top.chiloven.pulselink.java.pattern;

import java.util.Arrays;
import java.util.List;

/** The waveform data for one channel in a 100 ms frame. */
public sealed interface ChannelFrameSpec {

    static ChannelFrameSpec active(PulseSample... samples) {
        return new Active(Arrays.asList(samples));
    }

    static ChannelFrameSpec silent() {
        return new Silent();
    }

    /** Four 25 ms samples of active waveform data. */
    record Active(List<PulseSample> samples) implements ChannelFrameSpec {

        public Active {
            if (samples.size() != 4) {
                throw new IllegalArgumentException("active frame requires exactly 4 samples");
            }
            samples = List.copyOf(samples);
        }

    }

    /** Explicit silence for the whole 100 ms window. */
    record Silent() implements ChannelFrameSpec {

    }

}
