package top.chiloven.pulselink.core

/** The source of an intensity change. */
public enum class IntensitySource {

    Device,
    Sdk,
    PhysicalControl,
    Unknown,

}

/** A state change emitted by a device session. */
public sealed interface DeviceEvent {

    public data class ConnectionChanged(
        public val state: ConnectionState
    ) : DeviceEvent

    public data class BatteryChanged(
        public val battery: BatteryState
    ) : DeviceEvent

    public data class IntensityChanged(
        public val channel: Channel,
        public val intensity: Observed<Intensity>,
        public val source: IntensitySource,
    ) : DeviceEvent

    public data object SettingsChanged : DeviceEvent

    public data class PressureChanged(
        public val pressure: Observed<Pressure>,
    ) : DeviceEvent

    public data class ButtonChanged(
        public val buttons: ButtonMask
    ) : DeviceEvent

    public data class Triggered(
        public val trigger: Trigger
    ) : DeviceEvent

    public data object SensorData : DeviceEvent

    public data class Error(
        public val message: String
    ) : DeviceEvent

}
