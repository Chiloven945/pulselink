package top.chiloven.pulselink.java.transport;

import top.chiloven.pulselink.java.core.DeviceKind;

import java.util.List;
import java.util.UUID;

/** A local BLE device profile. */
public record DeviceProfile(
        DeviceKind kind,
        List<String> namePrefixes,
        List<UUID> serviceUuids
) {

    public DeviceProfile {
        namePrefixes = List.copyOf(namePrefixes);
        serviceUuids = List.copyOf(serviceUuids);
    }

}
