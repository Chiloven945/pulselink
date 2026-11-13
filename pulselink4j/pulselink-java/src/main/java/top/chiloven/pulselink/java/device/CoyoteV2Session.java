package top.chiloven.pulselink.java.device;

import top.chiloven.pulselink.java.core.Channel;

import java.util.concurrent.CompletableFuture;

/** A connected Coyote V2 session. */
public non-sealed interface CoyoteV2Session extends LocalSession {

    CompletableFuture<Void> setIntensity(Channel channel, int intensity);

    CompletableFuture<Void> setPulse(
            Channel channel,
            int x,
            int y,
            int z
    );

    CompletableFuture<Void> setPulseInterval(Channel channel, int milliseconds);

}
