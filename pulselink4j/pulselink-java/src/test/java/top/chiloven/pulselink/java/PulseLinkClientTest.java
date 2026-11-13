package top.chiloven.pulselink.java;

import org.junit.jupiter.api.Test;
import top.chiloven.pulselink.java.core.DeviceKind;
import top.chiloven.pulselink.java.device.CoyoteV3Session;
import top.chiloven.pulselink.java.device.LocalSession;
import top.chiloven.pulselink.java.pattern.ChannelFrameSpec;
import top.chiloven.pulselink.java.pattern.Pattern;
import top.chiloven.pulselink.java.pattern.PulseSample;
import top.chiloven.pulselink.java.pattern.StereoFrameSpec;
import top.chiloven.pulselink.java.runtime.CoyoteV3Config;
import top.chiloven.pulselink.java.transport.DeviceProfile;
import top.chiloven.pulselink.java.transport.DiscoveredDevice;
import top.chiloven.pulselink.java.transport.ScanFilter;

import java.time.Duration;
import java.util.List;
import java.util.concurrent.CompletableFuture;
import org.jspecify.annotations.NullMarked;

import static org.junit.jupiter.api.Assertions.*;

class PulseLinkClientTest {

    private static final DiscoveredDevice DEVICE = new DiscoveredDevice(
            "dev",
            "47L121000",
            -40,
            List.of(),
            List.of(),
            true
    );

    @Test
    void scanAndShutdownDelegateToBackend() {
        var client = new PulseLinkClient(new FakeBackend());

        assertEquals(
                List.of(DEVICE),
                client.scan(ScanFilter.all(), Duration.ofSeconds(1)).join()
        );
        client.shutdown().join();
    }

    @Test
    void patternBuilderCreatesStereoFrames() {
        var active = ChannelFrameSpec.active(
                new PulseSample(100, 20),
                new PulseSample(100, 30),
                new PulseSample(100, 20),
                new PulseSample(100, 10)
        );
        var pattern = Pattern.builder()
                .frame(new StereoFrameSpec(active, ChannelFrameSpec.silent()))
                .looped(true)
                .build();

        assertEquals(1, pattern.frames().size());
        assertTrue(pattern.looped());
    }

    @Test
    void deviceKindsSupportPatternMatching() {
        var kind = DeviceKind.fromRemoteType("COYOTE_030");
        assertInstanceOf(DeviceKind.CoyoteV3.class, kind);
        assertEquals("COYOTE_030", kind.remoteType());
        assertEquals("Coyote V3", kind.displayName());
    }

    @NullMarked
    private static final class FakeBackend implements PulseLinkBackend {

        @Override
        public CompletableFuture<List<DiscoveredDevice>> scan(
                ScanFilter filter,
                Duration timeout
        ) {
            return CompletableFuture.completedFuture(List.of(DEVICE));
        }

        @Override
        public CompletableFuture<List<DiscoveredDevice>> scanProfile(
                DeviceProfile profile,
                Duration timeout
        ) {
            return CompletableFuture.completedFuture(List.of(DEVICE));
        }

        @Override
        public CompletableFuture<LocalSession> connect(DiscoveredDevice device) {
            return CompletableFuture.failedFuture(
                    new UnsupportedOperationException("not used")
            );
        }

        @Override
        public CompletableFuture<CoyoteV3Session> connectCoyoteV3(
                DiscoveredDevice device,
                CoyoteV3Config config
        ) {
            return CompletableFuture.failedFuture(
                    new UnsupportedOperationException("not used")
            );
        }

        @Override
        public CompletableFuture<Void> shutdown() {
            return CompletableFuture.completedFuture(null);
        }

        @Override
        public void close() {
        }

    }

}
