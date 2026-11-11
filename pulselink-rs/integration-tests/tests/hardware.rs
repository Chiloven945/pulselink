//! Optional real-hardware smoke test.
//!
//! Run with `PULSELINK_HW=1` and a Coyote V3 nearby:
//! `cargo test -p pulselink-integration-tests --test hardware -- --ignored`

use std::sync::Arc;
use std::time::Duration;

use pulselink_ble::BtleplugCentral;
use pulselink_core::ConnectionState;
use pulselink_protocol::coyote::v3::gatt::{DEVICE_NAME, SERVICE_CONTROL};
use pulselink_runtime::{Client, CoyoteV3Config};
use pulselink_transport::ScanFilter;

#[tokio::test]
#[ignore = "requires PULSELINK_HW=1 and a real Coyote V3"]
async fn hardware_scan_connect_read_battery() {
    if std::env::var("PULSELINK_HW").as_deref() != Ok("1") {
        return;
    }

    let backend = Arc::new(BtleplugCentral::new().await.expect("adapter"));
    let client = Client::new(backend);
    let devices = client
        .scan(
            ScanFilter {
                name_prefixes: vec![DEVICE_NAME.to_owned()],
                service_uuids: vec![SERVICE_CONTROL],
            },
            Duration::from_secs(5),
        )
        .await
        .expect("scan");
    let device = devices.first().expect("no Coyote V3 found");
    let session = client
        .connect_coyote_v3(device, CoyoteV3Config::default())
        .await
        .expect("connect");
    assert_eq!(session.state().common.connection, ConnectionState::Ready);
    assert!(session.state().common.battery.is_known());
    session.shutdown().await.expect("shutdown");
    client.shutdown().await;
}
