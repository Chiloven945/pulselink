package top.chiloven.pulselink.device

import top.chiloven.pulselink.core.*

/** A local device event. */
public sealed interface LocalEvent {

    public data class ConnectionChanged(
        public val state: ConnectionState
    ) : LocalEvent

    public data class BatteryChanged(
        public val battery: BatteryState
    ) : LocalEvent

    public data class IntensityChanged(
        public val channel: Channel,
        public val intensity: Intensity
    ) : LocalEvent

    public data class PressureChanged(
        public val pressure: Pressure
    ) : LocalEvent

    public data class ButtonChanged(
        public val buttons: ButtonMask
    ) : LocalEvent

    public data class Triggered(
        public val trigger: Trigger
    ) : LocalEvent

    public data object SensorData : LocalEvent

    public data class Error(
        public val message: String
    ) : LocalEvent

    public data object Closed : LocalEvent

}
