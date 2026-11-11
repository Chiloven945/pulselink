#![cfg(feature = "ble")]
//! Facade-level smoke tests using the mock transport backend.

use std::sync::Arc;
use std::time::Duration;

use pulselink::PulseLink;
use pulselink::transport::mock::MockBackend;
use pulselink::transport::{DiscoveredDevice, ScanFilter, TransportDeviceId};

fn device(name: &str) -> DiscoveredDevice {
    DiscoveredDevice {
        id: TransportDeviceId::new("mock-1"),
        name: Some(name.to_owned()),
        rssi: Some(-40),
        service_uuids: Vec::new(),
        manufacturer_data: Vec::new(),
    }
}

#[tokio::test]
async fn scans_through_injected_backend() {
    let backend = Arc::new(MockBackend::new(vec![device("47L121000")]));
    let link = PulseLink::with_backend(backend);
    let devices = link
        .scan(
            ScanFilter {
                name_prefixes: vec!["47L".to_owned()],
                ..ScanFilter::default()
            },
            Duration::from_millis(10),
        )
        .await
        .expect("scan");
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].name.as_deref(), Some("47L121000"));
    link.shutdown().await;
}
