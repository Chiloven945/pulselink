package top.chiloven.pulselink.java.pattern;

import java.util.ArrayList;
import java.util.List;
import java.util.stream.Collectors;
import java.util.stream.IntStream;

/** A playable, optionally looping timeline. */
public final class Pattern {

    private final List<StereoFrameSpec> frames;
    private final boolean looped;

    private Pattern(List<StereoFrameSpec> frames, boolean looped) {
        if (frames.isEmpty()) {
            throw new IllegalArgumentException("pattern requires at least one frame");
        }
        this.frames = List.copyOf(frames);
        this.looped = looped;
    }

    public static Pattern of(List<StereoFrameSpec> frames, boolean looped) {
        return new Pattern(frames, looped);
    }

    public static Pattern silence(int frameCount) {
        if (frameCount <= 0) {
            throw new IllegalArgumentException("frameCount must be positive");
        }
        return new Pattern(
                IntStream.range(0, frameCount)
                        .mapToObj(i -> StereoFrameSpec.silent())
                        .collect(Collectors.toCollection(() -> new ArrayList<>(frameCount))),
                false
        );
    }

    public static Builder builder() {
        return new Builder();
    }

    public List<StereoFrameSpec> frames() {
        return frames;
    }

    public boolean looped() {
        return looped;
    }

    /** Builder for {@link Pattern}. */
    public static final class Builder {

        private final List<StereoFrameSpec> frames = new ArrayList<>();
        private boolean looped;

        private Builder() {
        }

        public Builder looped(boolean value) {
            this.looped = value;
            return this;
        }

        public Builder frame(StereoFrameSpec frame) {
            this.frames.add(frame);
            return this;
        }

        public Builder frames(List<StereoFrameSpec> frames) {
            this.frames.addAll(frames);
            return this;
        }

        public Builder silenceFrame() {
            this.frames.add(StereoFrameSpec.silent());
            return this;
        }

        public Pattern build() {
            return new Pattern(frames, looped);
        }

    }

}
