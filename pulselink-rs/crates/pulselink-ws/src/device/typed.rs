//! Typed views over the open V4 device cache.

use pulselink_core::{DeviceKind, Pressure};
use serde_json::Value;

use super::snapshot::RemoteDeviceSnapshot;

/// A typed view of a remote device snapshot.
#[derive(Clone, Debug)]
pub enum TypedRemoteSnapshot {
    /// Coyote V2 or V3.
    Coyote(CoyoteRemoteSnapshot),
    /// Opossum vibration controller.
    Opossum(OpossumRemoteSnapshot),
    /// Civet pressure sensor.
    Civet(CivetRemoteSnapshot),
    /// Unknown or future device type.
    Unknown(RemoteDeviceSnapshot),
}

impl RemoteDeviceSnapshot {
    /// Maps the V4 device type to a [`DeviceKind`].
    #[must_use]
    pub fn kind(&self) -> DeviceKind {
        DeviceKind::from_remote_type(&self.device_type)
    }

    /// Returns a typed view of this snapshot.
    #[must_use]
    pub fn typed(&self) -> TypedRemoteSnapshot {
        match self.kind() {
            DeviceKind::CoyoteV2 | DeviceKind::CoyoteV3 => {
                TypedRemoteSnapshot::Coyote(CoyoteRemoteSnapshot::new(self.clone()))
            }
            DeviceKind::Opossum => {
                TypedRemoteSnapshot::Opossum(OpossumRemoteSnapshot::new(self.clone()))
            }
            DeviceKind::Civet => TypedRemoteSnapshot::Civet(CivetRemoteSnapshot::new(self.clone())),
            _ => TypedRemoteSnapshot::Unknown(self.clone()),
        }
    }
}

/// Typed Coyote V2/V3 properties.
#[derive(Clone, Debug)]
pub struct CoyoteRemoteSnapshot {
    base: RemoteDeviceSnapshot,
}

impl CoyoteRemoteSnapshot {
    fn new(base: RemoteDeviceSnapshot) -> Self {
        Self { base }
    }

    /// Returns the underlying snapshot.
    #[must_use]
    pub fn snapshot(&self) -> &RemoteDeviceSnapshot {
        &self.base
    }

    /// Battery/power level.
    #[must_use]
    pub fn battery(&self) -> Option<u8> {
        self.base.battery()
    }

    /// Channel A intensity.
    #[must_use]
    pub fn intensity_a(&self) -> Option<u8> {
        self.base.intensity_a()
    }

    /// Channel B intensity.
    #[must_use]
    pub fn intensity_b(&self) -> Option<u8> {
        self.base.intensity_b()
    }

    /// Coyote channel A output status.
    #[must_use]
    pub fn channel_a_status(&self) -> Option<u8> {
        json_u8(self.base.props.get("channelAStatus"))
    }

    /// Coyote channel B output status.
    #[must_use]
    pub fn channel_b_status(&self) -> Option<u8> {
        json_u8(self.base.props.get("channelBStatus"))
    }

    /// Raw comfort-limit object for channel A.
    #[must_use]
    pub fn comfort_limit_a(&self) -> Option<&Value> {
        self.base
            .slot_state
            .get("channelA")
            .and_then(|value| value.get("comfortLimit"))
    }

    /// Raw comfort-limit object for channel B.
    #[must_use]
    pub fn comfort_limit_b(&self) -> Option<&Value> {
        self.base
            .slot_state
            .get("channelB")
            .and_then(|value| value.get("comfortLimit"))
    }
}

/// Typed Opossum properties.
#[derive(Clone, Debug)]
pub struct OpossumRemoteSnapshot {
    base: RemoteDeviceSnapshot,
}

impl OpossumRemoteSnapshot {
    fn new(base: RemoteDeviceSnapshot) -> Self {
        Self { base }
    }

    /// Returns the underlying snapshot.
    #[must_use]
    pub fn snapshot(&self) -> &RemoteDeviceSnapshot {
        &self.base
    }

    /// Battery/power level.
    #[must_use]
    pub fn battery(&self) -> Option<u8> {
        self.base.battery()
    }

    /// Channel A intensity.
    #[must_use]
    pub fn intensity_a(&self) -> Option<u8> {
        self.base.intensity_a()
    }

    /// Channel B intensity.
    #[must_use]
    pub fn intensity_b(&self) -> Option<u8> {
        self.base.intensity_b()
    }

    /// Opossum mode.
    #[must_use]
    pub fn mode(&self) -> Option<u8> {
        json_u8(self.base.props.get("mode"))
    }

    /// Whether channel A has an accessory connected.
    #[must_use]
    pub fn channel_a_connected(&self) -> Option<bool> {
        self.base
            .props
            .get("channelAStatus")
            .and_then(Value::as_bool)
    }

    /// Whether channel B has an accessory connected.
    #[must_use]
    pub fn channel_b_connected(&self) -> Option<bool> {
        self.base
            .props
            .get("channelBStatus")
            .and_then(Value::as_bool)
    }
}

/// Typed Civet properties.
#[derive(Clone, Debug)]
pub struct CivetRemoteSnapshot {
    base: RemoteDeviceSnapshot,
}

impl CivetRemoteSnapshot {
    fn new(base: RemoteDeviceSnapshot) -> Self {
        Self { base }
    }

    /// Returns the underlying snapshot.
    #[must_use]
    pub fn snapshot(&self) -> &RemoteDeviceSnapshot {
        &self.base
    }

    /// Battery/power level.
    #[must_use]
    pub fn battery(&self) -> Option<u8> {
        self.base.battery()
    }

    /// Pressure reading in kPa, when the app reports it.
    #[must_use]
    pub fn pressure(&self) -> Option<Pressure> {
        self.base
            .props
            .get("pressure")
            .and_then(Value::as_f64)
            .and_then(Pressure::from_kpa_f64)
    }

    /// Civet edging state.
    #[must_use]
    pub fn edge_state(&self) -> Option<u8> {
        self.base
            .slot_state
            .get("edge")
            .and_then(|value| value.get("edgeState"))
            .and_then(|value| json_u8(Some(value)))
    }
}

fn json_u8(value: Option<&Value>) -> Option<u8> {
    let value = value?;
    value
        .as_u64()
        .and_then(|value| u8::try_from(value).ok())
        .or_else(|| value.as_i64().and_then(|value| u8::try_from(value).ok()))
        .or_else(|| value.as_str().and_then(|value| value.parse().ok()))
}
