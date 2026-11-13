package top.chiloven.pulselink.java.device;

/** One event/probability pair for a PawPrints probability configuration. */
public record PawPrintsProbability(
        int eventId,
        int probability
) {

    public PawPrintsProbability {
        if (eventId < 0 || eventId > 255) {
            throw new IllegalArgumentException("eventId must be 0..255");
        }
        if (probability < 0 || probability > 100) {
            throw new IllegalArgumentException("probability must be 0..100");
        }
    }

}
