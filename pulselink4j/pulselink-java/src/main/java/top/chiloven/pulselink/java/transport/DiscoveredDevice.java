package top.chiloven.pulselink.java.transport;

import java.util.List;
import java.util.UUID;
import org.jspecify.annotations.Nullable;

/** A device discovered during a scan. */
public record DiscoveredDevice(
        String id,
        @Nullable String name,
        @Nullable Integer rssi,
        List<UUID> serviceUuids,
        List<ManufacturerData> manufacturerData,
        @Nullable Boolean connectable
) {

    public DiscoveredDevice {
        if (id.isBlank()) {
            throw new IllegalArgumentException("device id must not be blank");
        }
        serviceUuids = List.copyOf(serviceUuids);
        manufacturerData = List.copyOf(manufacturerData);
    }

}
