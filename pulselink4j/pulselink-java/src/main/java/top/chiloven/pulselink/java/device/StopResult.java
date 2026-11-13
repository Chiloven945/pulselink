package top.chiloven.pulselink.java.device;

/** The result of stopping output. */
public record StopResult(
        boolean acknowledged,
        int intensityA,
        int intensityB
) {

}
