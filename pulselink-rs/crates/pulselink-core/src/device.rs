//! Device kind and descriptive information.

use std::fmt;

use crate::id::DeviceId;

/// The kind of device known to `pulselink-rs`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum DeviceKind {
    /// Coyote stimulator, second generation.
    CoyoteV2,
    /// Coyote stimulator, third generation.
    CoyoteV3,
    /// `PawPrints` wireless button sensor.
    PawPrints,
    /// `Civet` edging sensor.
    Civet,
    /// `Opossum` vibration controller.
    Opossum,
    /// A device type not known to this version of `pulselink-rs`.
    Unknown(String),
}

impl DeviceKind {
    /// Maps a DG-LAB V4 app device type to a kind.
    #[must_use]
    pub fn from_remote_type(value: &str) -> Self {
        match value {
            "COYOTE_020" => Self::CoyoteV2,
            "COYOTE_030" => Self::CoyoteV3,
            "BMTR_1" => Self::Civet,
            "OVC_1" => Self::Opossum,
            other => Self::Unknown(other.to_owned()),
        }
    }

    /// Returns the DG-LAB V4 app device type, when one exists.
    #[must_use]
    pub const fn remote_type(&self) -> Option<&'static str> {
        match self {
            Self::CoyoteV2 => Some("COYOTE_020"),
            Self::CoyoteV3 => Some("COYOTE_030"),
            Self::Civet => Some("BMTR_1"),
            Self::Opossum => Some("OVC_1"),
            Self::PawPrints | Self::Unknown(_) => None,
        }
    }

    /// Returns a human-readable name for this device kind.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::CoyoteV2 => "Coyote V2",
            Self::CoyoteV3 => "Coyote V3",
            Self::PawPrints => "PawPrints",
            Self::Civet => "Civet",
            Self::Opossum => "Opossum",
            Self::Unknown(name) => name,
        }
    }
}

impl fmt::Display for DeviceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Descriptive information about a device.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeviceInfo {
    /// Stable identity within the current SDK session.
    pub id: DeviceId,
    /// Device kind.
    pub kind: DeviceKind,
    /// Human-readable device name.
    pub name: String,
    /// Manufacturer, when reported.
    pub manufacturer: Option<String>,
    /// Model, when reported.
    pub model: Option<String>,
    /// Firmware version, when reported.
    pub firmware: Option<String>,
}

impl DeviceInfo {
    /// Creates device information with the required fields.
    #[must_use]
    pub fn new(id: DeviceId, kind: DeviceKind, name: impl Into<String>) -> Self {
        Self {
            id,
            kind,
            name: name.into(),
            manufacturer: None,
            model: None,
            firmware: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_kind_display() {
        assert_eq!(DeviceKind::CoyoteV3.to_string(), "Coyote V3");
        assert_eq!(DeviceKind::Unknown("X".to_owned()).to_string(), "X");
    }

    #[test]
    fn device_info_builder() {
        let info = DeviceInfo::new(DeviceId::ble("p1"), DeviceKind::CoyoteV3, "Coyote");
        assert_eq!(info.name, "Coyote");
        assert_eq!(info.kind, DeviceKind::CoyoteV3);
    }

    #[test]
    fn remote_type_mapping() {
        assert_eq!(
            DeviceKind::from_remote_type("COYOTE_020"),
            DeviceKind::CoyoteV2
        );
        assert_eq!(
            DeviceKind::from_remote_type("COYOTE_030"),
            DeviceKind::CoyoteV3
        );
        assert_eq!(DeviceKind::from_remote_type("BMTR_1"), DeviceKind::Civet);
        assert_eq!(DeviceKind::from_remote_type("OVC_1"), DeviceKind::Opossum);
        assert_eq!(DeviceKind::CoyoteV2.remote_type(), Some("COYOTE_020"));
        assert_eq!(DeviceKind::PawPrints.remote_type(), None);
    }
}
