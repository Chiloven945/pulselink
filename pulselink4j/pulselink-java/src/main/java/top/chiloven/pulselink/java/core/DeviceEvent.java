package top.chiloven.pulselink.java.core;

import org.jspecify.annotations.Nullable;

/** A state change emitted by a device session. */
public sealed interface DeviceEvent {

    record ConnectionChanged(ConnectionState state) implements DeviceEvent {

    }

    record BatteryChanged(BatteryState battery) implements DeviceEvent {

    }

    record IntensityChanged(
            Channel channel,
            @Nullable Integer intensity,
            IntensitySource source
    ) implements DeviceEvent {

    }

    record SettingsChanged() implements DeviceEvent {

    }

    record PressureChanged(@Nullable Pressure pressure) implements DeviceEvent {

    }

    record ButtonChanged(ButtonMask buttons) implements DeviceEvent {

    }

    record Triggered(Trigger trigger) implements DeviceEvent {

    }

    record SensorData() implements DeviceEvent {

    }

    record Failure(String message) implements DeviceEvent {

    }

}
