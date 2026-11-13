package top.chiloven.pulselink.java.runtime;

/** Coyote V3 balance parameters. */
public record CoyoteV3Balance(
        int frequencyA,
        int frequencyB,
        int intensityA,
        int intensityB
) {

    public CoyoteV3Balance {
        requireBalance("frequencyA", frequencyA);
        requireBalance("frequencyB", frequencyB);
        requireBalance("intensityA", intensityA);
        requireBalance("intensityB", intensityB);
    }

    private static void requireBalance(String name, int value) {
        if (value < 0 || value > 255) {
            throw new IllegalArgumentException(name + " must be 0..255");
        }
    }

    public static CoyoteV3Balance defaults() {
        return new CoyoteV3Balance(
                128,
                128,
                128,
                128
        );
    }

}
