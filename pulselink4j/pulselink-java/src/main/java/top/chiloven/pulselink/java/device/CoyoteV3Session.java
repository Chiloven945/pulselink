package top.chiloven.pulselink.java.device;

import top.chiloven.pulselink.java.core.Channel;
import top.chiloven.pulselink.java.pattern.Pattern;
import top.chiloven.pulselink.java.runtime.CoyoteV3Balance;
import top.chiloven.pulselink.java.runtime.CoyoteV3Event;
import top.chiloven.pulselink.java.runtime.CoyoteV3State;
import top.chiloven.pulselink.java.runtime.PlayOptions;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.Flow;

/** A connected Coyote V3 session. */
public non-sealed interface CoyoteV3Session extends LocalSession {

    CoyoteV3State runtimeState();

    Flow.Publisher<CoyoteV3State> runtimeStates();

    Flow.Publisher<CoyoteV3Event> runtimeEvents();

    CompletableFuture<Void> setIntensity(Channel channel, int intensity);

    CompletableFuture<Void> adjustIntensity(Channel channel, int delta);

    CompletableFuture<Void> setSoftLimit(Channel channel, int intensity);

    CompletableFuture<Void> setBalance(CoyoteV3Balance balance);

    CompletableFuture<PlayHandle> play(Pattern pattern);

    CompletableFuture<PlayHandle> play(Pattern pattern, PlayOptions options);

    CompletableFuture<StopResult> stop();

    CompletableFuture<Void> reconnect();

}
