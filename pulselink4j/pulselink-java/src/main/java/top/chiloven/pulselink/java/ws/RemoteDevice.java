package top.chiloven.pulselink.java.ws;

import top.chiloven.pulselink.java.core.DeviceKind;

import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.Map;
import org.jspecify.annotations.Nullable;

/** A device exposed by a DG-LAB V4 app client. */
public record RemoteDevice(
        @Nullable Integer id,
        String slotId,
        String name,
        String deviceType,
        Map<String, @Nullable Object> props,
        Map<String, @Nullable Object> slotState
) {

    public RemoteDevice {
        props = Collections.unmodifiableMap(new LinkedHashMap<>(props));
        slotState = Collections.unmodifiableMap(new LinkedHashMap<>(slotState));
    }

    public DeviceKind kind() {
        return DeviceKind.fromRemoteType(deviceType);
    }

    public TypedRemoteDevice typed() {
        return switch (deviceType) {
            case "COYOTE_020", "COYOTE_030" -> new TypedRemoteDevice.Coyote(this);
            case "BMTR_1" -> new TypedRemoteDevice.Civet(this);
            case "OVC_1" -> new TypedRemoteDevice.Opossum(this);
            default -> new TypedRemoteDevice.Unknown(this);
        };
    }

}
