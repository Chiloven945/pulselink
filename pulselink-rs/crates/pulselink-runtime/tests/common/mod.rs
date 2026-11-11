//! Shared runtime test helpers.

use std::time::Duration;

use pulselink_runtime::CoyoteV3Config;
use pulselink_transport::{DiscoveredDevice, TransportDeviceId};

pub(crate) fn device() -> DiscoveredDevice {
    DiscoveredDevice {
        id: TransportDeviceId::new("coyote-1"),
        name: Some("47L121000".to_owned()),
        rssi: Some(-40),
        service_uuids: Vec::new(),
        manufacturer_data: Vec::new(),
    }
}

pub(crate) fn config() -> CoyoteV3Config {
    CoyoteV3Config {
        ack_timeout: Duration::from_millis(10),
        ..CoyoteV3Config::default()
    }
}
