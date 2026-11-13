package top.chiloven.pulselink.java.runtime;

/** Why pattern playback ended. */
public enum PlayEndReason {

    COMPLETED,
    STOPPED,
    REPLACED,
    SCHEDULER_LAG,
    LINK_LOST,
    SHUTDOWN,
    FAILED

}
