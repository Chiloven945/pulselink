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

interface PulseLinkBackend extends AutoCloseable {

    CompletableFuture<List<DiscoveredDevice>> scan(
            ScanFilter filter,
            Duration timeout
    );

    CompletableFuture<List<DiscoveredDevice>> scanProfile(
            DeviceProfile profile,
            Duration timeout
    );

    CompletableFuture<LocalSession> connect(DiscoveredDevice device);

    CompletableFuture<CoyoteV3Session> connectCoyoteV3(
            DiscoveredDevice device,
            CoyoteV3Config config
    );

    CompletableFuture<Void> shutdown();

    @Override
    void close();

}
