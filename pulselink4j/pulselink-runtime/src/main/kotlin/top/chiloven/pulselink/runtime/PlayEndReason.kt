package top.chiloven.pulselink.runtime

/** Why pattern playback ended. */
public enum class PlayEndReason {

    Completed,
    Stopped,
    Replaced,
    SchedulerLag,
    LinkLost,
    Shutdown,
    Failed,

}
