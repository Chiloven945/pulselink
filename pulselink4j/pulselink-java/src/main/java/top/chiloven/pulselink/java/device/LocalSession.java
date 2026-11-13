package top.chiloven.pulselink.java.device;

import top.chiloven.pulselink.java.core.DeviceKind;
import top.chiloven.pulselink.java.core.DeviceState;
import top.chiloven.pulselink.java.core.LocalEvent;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.Flow;

/** A connected local device session. */
public sealed interface LocalSession extends AutoCloseable
        permits CoyoteV2Session, CoyoteV3Session, OpossumSession, CivetSession, PawPrintsSession {

    DeviceKind kind();

    DeviceState state();

    Flow.Publisher<DeviceState> states();

    Flow.Publisher<LocalEvent> events();

    CompletableFuture<Void> shutdown();

    @Override
    void close();

}
