package top.chiloven.pulselink.java.ws;

import java.util.List;

/** The {@code v} field of a device operation. */
public sealed interface OperateValue {

    record Intensity(int value) implements OperateValue {

    }

    record PulseFrames(List<byte[]> frames) implements OperateValue {

        public PulseFrames {
            frames = frames.stream()
                    .map(byte[]::clone)
                    .toList();
        }

        @Override
        public List<byte[]> frames() {
            return frames.stream()
                    .map(byte[]::clone)
                    .toList();
        }

    }

}
