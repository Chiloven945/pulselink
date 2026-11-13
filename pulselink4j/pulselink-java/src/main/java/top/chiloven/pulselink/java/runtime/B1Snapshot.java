package top.chiloven.pulselink.java.runtime;

/** The last decoded B1 intensity notification. */
public record B1Snapshot(
        int sequence,
        int intensityA,
        int intensityB
) {

}
