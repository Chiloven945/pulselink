package top.chiloven.pulselink.java.core;

/** A parameterized trigger event reported by a sensor. */
public record Trigger(
        int eventId,
        int parameter
) {

    public Trigger {
        if (eventId < 0 || eventId > 255 || parameter < 0 || parameter > 255) {
            throw new IllegalArgumentException("trigger fields must be 0..255");
        }
    }

}
