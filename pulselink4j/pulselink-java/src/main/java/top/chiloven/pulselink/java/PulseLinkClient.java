package top.chiloven.pulselink.java;

import top.chiloven.pulselink.java.device.CoyoteV3Session;
import top.chiloven.pulselink.java.device.LocalSession;
import top.chiloven.pulselink.java.runtime.CoyoteV3Config;
import top.chiloven.pulselink.java.transport.DeviceProfile;
import top.chiloven.pulselink.java.transport.DiscoveredDevice;
import top.chiloven.pulselink.java.transport.ScanFilter;

import java.time.Duration;
import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicBoolean;

/** The high-level local-device entry point. */
public final class PulseLinkClient implements AutoCloseable {

    private final PulseLinkBackend backend;
    private final AtomicBoolean closed = new AtomicBoolean();

    public PulseLinkClient() {
        this.backend = BridgeFactory.createClient();
    }

    PulseLinkClient(PulseLinkBackend backend) {
        this.backend = backend;
    }

    public CompletableFuture<List<DiscoveredDevice>> scan(
            ScanFilter filter,
            Duration timeout
    ) {
        checkOpen();
        return backend.scan(filter, timeout);
    }

    private void checkOpen() {
        if (closed.get()) {
            throw new PulseLinkClosedException();
        }
    }

    public CompletableFuture<List<DiscoveredDevice>> scanProfile(
            DeviceProfile profile,
            Duration timeout
    ) {
        checkOpen();
        return backend.scanProfile(profile, timeout);
    }

    public CompletableFuture<LocalSession> connect(DiscoveredDevice device) {
        checkOpen();
        return backend.connect(device);
    }

    public CompletableFuture<CoyoteV3Session> connectCoyoteV3(DiscoveredDevice device) {
        return connectCoyoteV3(device, CoyoteV3Config.defaults());
    }

    public CompletableFuture<CoyoteV3Session> connectCoyoteV3(
            DiscoveredDevice device,
            CoyoteV3Config config
    ) {
        checkOpen();
        return backend.connectCoyoteV3(device, config);
    }

    public CompletableFuture<Void> shutdown() {
        return !closed.compareAndSet(false, true)
                ? CompletableFuture.completedFuture(null)
                : backend.shutdown();
    }

    @Override
    public void close() {
        if (!closed.compareAndSet(false, true)) {
            return;
        }
        backend.close();
    }

}
