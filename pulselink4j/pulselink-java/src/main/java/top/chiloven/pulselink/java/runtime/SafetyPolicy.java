package top.chiloven.pulselink.java.runtime;

import org.jspecify.annotations.Nullable;

/** Safety limits applied before intensity operations. */
public record SafetyPolicy(
        int maxIntensityA,
        int maxIntensityB,
        @Nullable Integer maxDeltaA,
        @Nullable Integer maxDeltaB,
        boolean allowTemporary,
        boolean requireSoftLimit
) {

    public SafetyPolicy {
        requireIntensity("maxIntensityA", maxIntensityA);
        requireIntensity("maxIntensityB", maxIntensityB);
        requireDelta("maxDeltaA", maxDeltaA);
        requireDelta("maxDeltaB", maxDeltaB);
    }

    private static void requireIntensity(String name, int value) {
        if (value < 0 || value > 200) {
            throw new IllegalArgumentException(name + " must be 0..200");
        }
    }

    private static void requireDelta(String name, @Nullable Integer value) {
        if (value != null && (value < -200 || value > 200)) {
            throw new IllegalArgumentException(name + " must be -200..200");
        }
    }

    public static SafetyPolicy defaults() {
        return new SafetyPolicy(
                200,
                200,
                null,
                null,
                true,
                false
        );
    }

}
