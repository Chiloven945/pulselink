package top.chiloven.pulselink.java.core;

import org.jspecify.annotations.Nullable;

/** A local device event. */
public sealed interface LocalEvent {

    record ConnectionChanged(ConnectionState state) implements LocalEvent {

    }

    record BatteryChanged(BatteryState battery) implements LocalEvent {

    }

    record IntensityChanged(
            Channel channel,
            @Nullable Integer intensity
    ) implements LocalEvent {

    }

    record PressureChanged(@Nullable Pressure pressure) implements LocalEvent {

    }

    record ButtonChanged(ButtonMask buttons) implements LocalEvent {

    }

    record Triggered(Trigger trigger) implements LocalEvent {

    }

    record SensorData() implements LocalEvent {

    }

    record Failure(String message) implements LocalEvent {

    }

    record Closed() implements LocalEvent {

    }

}
