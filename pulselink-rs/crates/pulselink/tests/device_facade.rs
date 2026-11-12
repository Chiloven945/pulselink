//! Facade integration for the new local-device drivers.
#![cfg(feature = "ble")]

use std::sync::Arc;
use std::time::Duration;

use pulselink::device::COYOTE_V2;
use pulselink::protocol::coyote::v2::SERVICE_PWM;
use pulselink::transport::mock::{MockBackend, MockLink};
use pulselink::transport::{DiscoveredDevice, TransportDeviceId};
use pulselink::{LocalSession, PulseLink};

#[tokio::test]
async fn facade_connects_v2_through_device_client() {
    let device = DiscoveredDevice {
        id: TransportDeviceId::new("v2"),
        name: Some("D-LAB ESTIM01".to_owned()),
        rssi: Some(-40),
        service_uuids: vec![SERVICE_PWM],
        manufacturer_data: Vec::new(),
    };
    let link = MockLink::new(device.id.clone());
    let backend = MockBackend::new(vec![device.clone()]);
    backend.push_link(link);
    let facade = PulseLink::with_backend(Arc::new(backend));

    let devices = facade
        .scan_profile(&COYOTE_V2, Duration::from_secs(1))
        .await
        .expect("scan");
    assert_eq!(devices.len(), 1);
    let session = facade.connect_device(&device).await.expect("connect");
    assert!(matches!(session, LocalSession::CoyoteV2(_)));
    session.shutdown().await.expect("shutdown");
}
