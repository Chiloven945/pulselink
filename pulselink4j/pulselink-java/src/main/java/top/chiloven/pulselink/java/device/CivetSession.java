package top.chiloven.pulselink.java.device;

import java.util.concurrent.CompletableFuture;

/** A connected Civet session. */
public non-sealed interface CivetSession extends LocalSession {

    CompletableFuture<Void> startPressure(int color);

    CompletableFuture<Void> stopPressure(int color);

    CompletableFuture<Void> resetPressure();

    CompletableFuture<Void> setOrientation(int value);

}
