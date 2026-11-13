package top.chiloven.pulselink.java.transport;

import java.util.Arrays;
import java.util.List;
import java.util.UUID;

/** A filter applied during scanning and reconnection. */
public record ScanFilter(
        List<String> namePrefixes,
        List<UUID> serviceUuids
) {

    public ScanFilter {
        namePrefixes = List.copyOf(namePrefixes);
        serviceUuids = List.copyOf(serviceUuids);
    }

    /** A filter that matches every device. */
    public static ScanFilter all() {
        return new ScanFilter(List.of(), List.of());
    }

    /** A filter that matches any of the given name prefixes. */
    public static ScanFilter namePrefix(String... prefixes) {
        return new ScanFilter(Arrays.asList(prefixes), List.of());
    }

    /** A filter that matches any of the given service UUIDs. */
    public static ScanFilter serviceUuid(UUID... uuids) {
        return new ScanFilter(List.of(), Arrays.asList(uuids));
    }

}
