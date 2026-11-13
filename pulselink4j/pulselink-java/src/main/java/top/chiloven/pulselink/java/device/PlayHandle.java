package top.chiloven.pulselink.java.device;

import top.chiloven.pulselink.java.runtime.PlayEndReason;

import java.util.concurrent.CompletableFuture;

/** A handle for one pattern playback. */
public interface PlayHandle {

    long id();

    boolean isFinished();

    CompletableFuture<PlayEndReason> completion();

    CompletableFuture<PlayEndReason> stop();

}
