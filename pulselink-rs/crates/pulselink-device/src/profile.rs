//! Device discovery profiles.

use pulselink_core::{DeviceKind, DeviceState};
use pulselink_protocol::civet as civet_protocol;
use pulselink_protocol::coyote::v2 as coyote_v2_protocol;
use pulselink_protocol::coyote::v3::gatt as coyote_v3_protocol;
use pulselink_protocol::opossum as opossum_protocol;
use pulselink_protocol::paw_prints as paw_protocol;
use pulselink_transport::{DiscoveredDevice, ScanFilter};

/// A local device discovery profile.
#[derive(Clone, Debug)]
pub struct DeviceProfile {
    /// Device kind.
    pub kind: DeviceKind,
    /// Accepted BLE name prefixes.
    pub name_prefixes: &'static [&'static str],
    /// Accepted service UUIDs.
    pub service_uuids: &'static [uuid::Uuid],
}

impl DeviceProfile {
    /// Returns the primary name-based scan filter for this profile.
    ///
    /// Name prefixes are used as the primary filter. Passing service UUIDs to
    /// the platform BLE scanner can hide devices that do not advertise the
    /// service UUID in every advertising packet. `DeviceClient` still uses the
    /// service UUIDs for classification.
    #[must_use]
    pub fn scan_filter(&self) -> ScanFilter {
        ScanFilter {
            name_prefixes: self
                .name_prefixes
                .iter()
                .map(|prefix| (*prefix).to_owned())
                .collect(),
            service_uuids: Vec::new(),
        }
    }

    /// Returns a service-UUID fallback for devices that no longer advertise a
    /// local name.
    ///
    /// Recent Coyote V3 firmware can use a privacy-broadcast mode that omits
    /// the `47L121000` local name while keeping the `0x180C` GATT service. This
    /// filter finds such devices; callers should still verify the profile after
    /// connecting.
    #[must_use]
    pub fn fallback_scan_filter(&self) -> ScanFilter {
        ScanFilter {
            name_prefixes: Vec::new(),
            service_uuids: self.service_uuids.to_vec(),
        }
    }

    /// Returns whether a discovered device belongs to this profile.
    ///
    /// Name matches take priority. An unnamed device advertising the shared
    /// `0x180C` service is treated as a Coyote V3 candidate because V3 is the
    /// currently documented device with a privacy-broadcast mode.
    #[must_use]
    pub fn matches(&self, device: &DiscoveredDevice) -> bool {
        classify(device) == self.kind
            || (self.kind == DeviceKind::CoyoteV3
                && device.name.is_none()
                && device
                    .service_uuids
                    .contains(&coyote_v3_protocol::SERVICE_CONTROL))
    }

    /// Returns a fresh common state for this device.
    #[must_use]
    pub fn initial_state(&self) -> DeviceState {
        DeviceState::new(pulselink_core::ConnectionState::Initializing)
    }
}

const COYOTE_V2_SERVICES: &[uuid::Uuid] = &[coyote_v2_protocol::SERVICE_PWM];
const COYOTE_V3_SERVICES: &[uuid::Uuid] = &[coyote_v3_protocol::SERVICE_CONTROL];
const OPOSSUM_SERVICES: &[uuid::Uuid] = &[opossum_protocol::SERVICE_CONTROL];
const CIVET_SERVICES: &[uuid::Uuid] = &[civet_protocol::SERVICE_CONTROL];
const PAW_SERVICES: &[uuid::Uuid] = &[paw_protocol::SERVICE_CONTROL];

/// Coyote V2 profile.
pub const COYOTE_V2: DeviceProfile = DeviceProfile {
    kind: DeviceKind::CoyoteV2,
    name_prefixes: &[coyote_v2_protocol::DEVICE_NAME],
    service_uuids: COYOTE_V2_SERVICES,
};
/// Coyote V3 profile.
pub const COYOTE_V3: DeviceProfile = DeviceProfile {
    kind: DeviceKind::CoyoteV3,
    name_prefixes: &[coyote_v3_protocol::DEVICE_NAME],
    service_uuids: COYOTE_V3_SERVICES,
};
/// Opossum profile.
pub const OPOSSUM: DeviceProfile = DeviceProfile {
    kind: DeviceKind::Opossum,
    name_prefixes: &[opossum_protocol::DEVICE_NAME],
    service_uuids: OPOSSUM_SERVICES,
};
/// Civet profile.
pub const CIVET: DeviceProfile = DeviceProfile {
    kind: DeviceKind::Civet,
    name_prefixes: &[civet_protocol::DEVICE_NAME],
    service_uuids: CIVET_SERVICES,
};
/// `PawPrints` profile.
pub const PAW_PRINTS: DeviceProfile = DeviceProfile {
    kind: DeviceKind::PawPrints,
    name_prefixes: &[
        paw_protocol::DEVICE_NAME_V1_1,
        paw_protocol::DEVICE_NAME_V1_0,
    ],
    service_uuids: PAW_SERVICES,
};

/// All built-in local device profiles.
pub const PROFILES: &[DeviceProfile] = &[COYOTE_V2, COYOTE_V3, OPOSSUM, CIVET, PAW_PRINTS];

/// Classifies a discovered device by name first, then service UUID.
#[must_use]
pub fn classify(device: &DiscoveredDevice) -> DeviceKind {
    if let Some(name) = device.name.as_deref() {
        for profile in PROFILES {
            if profile
                .name_prefixes
                .iter()
                .any(|prefix| name.starts_with(prefix))
            {
                return profile.kind.clone();
            }
        }
    }

    for profile in PROFILES {
        if profile
            .service_uuids
            .iter()
            .any(|service| device.service_uuids.contains(service))
        {
            return profile.kind.clone();
        }
    }

    DeviceKind::Unknown(
        device
            .name
            .clone()
            .unwrap_or_else(|| "unknown BLE device".to_owned()),
    )
}

/// Looks up a built-in profile by device kind.
#[must_use]
pub fn profile_for(kind: &DeviceKind) -> Option<&'static DeviceProfile> {
    PROFILES.iter().find(|profile| &profile.kind == kind)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulselink_transport::TransportDeviceId;

    fn device(name: &str, service: uuid::Uuid) -> DiscoveredDevice {
        DiscoveredDevice {
            id: TransportDeviceId::new("dev"),
            name: Some(name.to_owned()),
            rssi: None,
            service_uuids: vec![service],
            manufacturer_data: Vec::new(),
        }
    }

    #[test]
    fn classifies_by_name_before_shared_service() {
        assert_eq!(
            classify(&device("47L127000", coyote_v3_protocol::SERVICE_CONTROL)),
            DeviceKind::Opossum
        );
        assert_eq!(
            classify(&device("47L124000", coyote_v3_protocol::SERVICE_CONTROL)),
            DeviceKind::Civet
        );
        assert_eq!(
            classify(&device("47L120300", coyote_v3_protocol::SERVICE_CONTROL)),
            DeviceKind::PawPrints
        );
    }

    #[test]
    fn classifies_v2_by_service() {
        assert_eq!(
            classify(&device("D-LAB ESTIM01", coyote_v2_protocol::SERVICE_PWM)),
            DeviceKind::CoyoteV2
        );
    }

    fn unnamed_v3() -> DiscoveredDevice {
        DiscoveredDevice {
            id: TransportDeviceId::new("aa:bb:cc:dd:ee:ff"),
            name: None,
            rssi: Some(-50),
            service_uuids: vec![coyote_v3_protocol::SERVICE_CONTROL],
            manufacturer_data: Vec::new(),
        }
    }

    #[test]
    fn fallback_filter_uses_service_uuid() {
        let filter = COYOTE_V3.fallback_scan_filter();
        assert_eq!(filter.name_prefixes, Vec::<String>::new());
        assert_eq!(
            filter.service_uuids,
            vec![coyote_v3_protocol::SERVICE_CONTROL]
        );
    }

    #[test]
    fn matches_unnamed_v3_privacy_broadcast() {
        assert!(COYOTE_V3.matches(&unnamed_v3()));
        assert_eq!(classify(&unnamed_v3()), DeviceKind::CoyoteV3);
    }

    #[test]
    fn named_shared_service_is_not_a_v3_match() {
        let opossum = device("47L127000", coyote_v3_protocol::SERVICE_CONTROL);
        assert!(!COYOTE_V3.matches(&opossum));
        assert!(OPOSSUM.matches(&opossum));
    }
}
