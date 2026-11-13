package top.chiloven.pulselink.java.transport;

import top.chiloven.pulselink.java.core.DeviceKind;

import java.util.List;
import java.util.UUID;

/** Built-in local device profiles. */
public final class PulseLinkProfiles {

    private static final UUID CONTROL = UUID.fromString(
            "0000180c-0000-1000-8000-00805f9b34fb"
    );

    public static final DeviceProfile COYOTE_V3 = new DeviceProfile(
            new DeviceKind.CoyoteV3(),
            List.of("47L121000"),
            List.of(CONTROL)
    );
    public static final DeviceProfile OPOSSUM = new DeviceProfile(
            new DeviceKind.Opossum(),
            List.of("47L127000"),
            List.of(CONTROL)
    );
    public static final DeviceProfile CIVET = new DeviceProfile(
            new DeviceKind.Civet(),
            List.of("47L124000"),
            List.of(CONTROL)
    );
    public static final DeviceProfile PAW_PRINTS = new DeviceProfile(
            new DeviceKind.PawPrints(),
            List.of("47L120300", "47L120100"),
            List.of(CONTROL)
    );
    private static final UUID PWM =
            UUID.fromString("0000180b-0000-1000-8000-00805f9b34fb");
    public static final DeviceProfile COYOTE_V2 = new DeviceProfile(
            new DeviceKind.CoyoteV2(),
            List.of("D-LAB ESTIM01"),
            List.of(PWM)
    );

    private PulseLinkProfiles() {
    }

    public static List<DeviceProfile> all() {
        return List.of(COYOTE_V2, COYOTE_V3, OPOSSUM, CIVET, PAW_PRINTS);
    }

}
