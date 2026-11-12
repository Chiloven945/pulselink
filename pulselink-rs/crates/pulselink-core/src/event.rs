//! Device events emitted by sessions.

use crate::button::{ButtonMask, Trigger};
use crate::channel::Channel;
use crate::intensity::Intensity;
use crate::pressure::Pressure;
use crate::state::{BatteryState, ConnectionState, Observed};

/// The source of an intensity change.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IntensitySource {
    /// The device reported the change on its own (for example, a physical wheel).
    Device,
    /// The SDK applied the change.
    Sdk,
    /// A physical control on the device.
    PhysicalControl,
    /// The source could not be determined.
    Unknown,
}

/// A state change emitted by a device session.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum DeviceEvent {
    /// The connection state changed.
    ConnectionChanged {
        /// The new connection state.
        state: ConnectionState,
    },
    /// The battery state changed.
    BatteryChanged {
        /// The new battery state.
        battery: BatteryState,
    },
    /// A channel intensity changed.
    IntensityChanged {
        /// The affected channel.
        channel: Channel,
        /// The observed intensity.
        intensity: Observed<Intensity>,
        /// The source of the change.
        source: IntensitySource,
    },
    /// Device settings changed or were written.
    SettingsChanged,
    /// A pressure sensor value changed.
    PressureChanged {
        /// Observed pressure.
        pressure: Observed<Pressure>,
    },
    /// Physical button state changed.
    ButtonChanged {
        /// Pressed-button mask.
        buttons: ButtonMask,
    },
    /// A sensor trigger event fired.
    Trigger {
        /// Trigger details.
        trigger: Trigger,
    },
    /// Raw sensor data was received.
    SensorData,
    /// A non-fatal or fatal error occurred.
    Error {
        /// Human-readable error message.
        message: String,
    },
}
