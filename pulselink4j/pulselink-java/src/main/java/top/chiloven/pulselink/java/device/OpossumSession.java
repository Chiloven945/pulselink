package top.chiloven.pulselink.java.device;

import top.chiloven.pulselink.java.core.Channel;

import java.util.concurrent.CompletableFuture;

/** A connected Opossum session. */
public non-sealed interface OpossumSession extends LocalSession {

    CompletableFuture<Void> setIntensity(Channel channel, int intensity);

    CompletableFuture<Void> pulse(Channel channel, int[] values);

    CompletableFuture<Void> display(int a, int b);

}
