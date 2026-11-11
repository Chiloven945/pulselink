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
}
