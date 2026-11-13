package top.chiloven.pulselink.java.core;

/** A local or remote device identity. */
public sealed interface DeviceId {

    record Ble(String value) implements DeviceId {

    }

    record Remote(
            String clientId,
            String slotId
    ) implements DeviceId {

    }

}
