//! Discovery fallback tests for unnamed privacy-broadcast devices.

use std::sync::Arc;
use std::time::Duration;

use pulselink_device::{COYOTE_V3, DeviceClient};
use pulselink_protocol::coyote::v3::gatt::SERVICE_CONTROL;
use pulselink_transport::mock::MockBackend;
use pulselink_transport::{Backend, DiscoveredDevice, TransportDeviceId};

fn unnamed_coyote() -> DiscoveredDevice {
    DiscoveredDevice {
        id: TransportDeviceId::new("aa:bb:cc:dd:ee:ff"),
        name: None,
        rssi: Some(-50),
        service_uuids: vec![SERVICE_CONTROL],
        manufacturer_data: Vec::new(),
    }
}

fn named_opossum() -> DiscoveredDevice {
    DiscoveredDevice {
        id: TransportDeviceId::new("11:22:33:44:55:66"),
        name: Some("47L127000".to_owned()),
        rssi: Some(-40),
        service_uuids: vec![SERVICE_CONTROL],
        manufacturer_data: Vec::new(),
    }
}

#[tokio::test]
async fn profile_scan_falls_back_to_service_uuid() {
    let backend: Arc<dyn Backend> = Arc::new(MockBackend::new(vec![unnamed_coyote()]));
    let client = DeviceClient::new(backend);
    let devices = client
        .scan_profile(&COYOTE_V3, Duration::from_millis(10))
        .await
        .expect("scan");
    assert_eq!(devices.len(), 1);
    assert!(devices[0].name.is_none());
    assert_eq!(devices[0].service_uuids, vec![SERVICE_CONTROL]);
}

#[tokio::test]
async fn fallback_does_not_misclassify_named_opossum() {
    let backend: Arc<dyn Backend> = Arc::new(MockBackend::new(vec![named_opossum()]));
    let client = DeviceClient::new(backend);
    let devices = client
        .scan_profile(&COYOTE_V3, Duration::from_millis(10))
        .await
        .expect("scan");
    assert!(devices.is_empty());
}

#[test]
fn service_uuid_matches_unnamed_device() {
    let filter = COYOTE_V3.fallback_scan_filter();
    assert!(filter.matches(&unnamed_coyote()));
}
