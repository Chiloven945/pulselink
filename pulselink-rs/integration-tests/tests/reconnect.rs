//! Runtime reconnect scenarios.

use std::sync::Arc;
use std::time::Duration;

use pulselink_core::ConnectionState;
use pulselink_protocol::coyote::v3::gatt::{CHAR_BATTERY, CHAR_WRITE};
use pulselink_runtime::{Client, ClientConfig, CoyoteV3Config};
use pulselink_transport::mock::{MockBackend, MockLink};
use pulselink_transport::{DiscoveredDevice, TransportDeviceId};

fn device() -> DiscoveredDevice {
    DiscoveredDevice {
        id: TransportDeviceId::new("coyote-1"),
        name: Some("47L121000".to_owned()),
        rssi: None,
        service_uuids: Vec::new(),
        manufacturer_data: Vec::new(),
    }
}

#[tokio::test(start_paused = true)]
async fn reconnect_reinitializes_and_rewrites_bf() {
    let first = MockLink::new(TransportDeviceId::new("coyote-1"));
    let second = MockLink::new(TransportDeviceId::new("coyote-1"));
    first.set_read_data(CHAR_BATTERY, vec![80]);
    second.set_read_data(CHAR_BATTERY, vec![79]);
    let backend = Arc::new(MockBackend::new(vec![device()]));
    backend.push_link(first.clone());
    backend.push_link(second.clone());
    let client = Client::with_config(backend, ClientConfig::default());
    let session = client
        .connect_coyote_v3(
            &device(),
            CoyoteV3Config {
                ack_timeout: Duration::from_millis(10),
                ..CoyoteV3Config::default()
            },
        )
        .await
        .expect("connect");

    first.mark_disconnected();
    tokio::task::yield_now().await;
    assert!(matches!(
        session.state().common.connection,
        ConnectionState::Degraded | ConnectionState::Reconnecting
    ));

    tokio::time::advance(Duration::from_millis(500)).await;
    for _ in 0..20 {
        tokio::task::yield_now().await;
    }
    tokio::time::advance(Duration::from_millis(10)).await;
    for _ in 0..20 {
        tokio::task::yield_now().await;
    }
    assert_eq!(session.state().common.connection, ConnectionState::Ready);
    assert!(
        second
            .writes()
            .iter()
            .any(|write| write.characteristic == CHAR_WRITE && write.data[0] == 0xBF)
    );
}

#[tokio::test(start_paused = true)]
async fn reconnect_failure_enters_disconnected() {
    let first = MockLink::new(TransportDeviceId::new("coyote-1"));
    first.set_read_data(CHAR_BATTERY, vec![80]);
    let backend = Arc::new(MockBackend::new(vec![device()]));
    backend.push_link(first.clone());
    let client = Client::with_config(backend, ClientConfig::default());
    let config = CoyoteV3Config {
        ack_timeout: Duration::from_millis(10),
        reconnect: pulselink_runtime::ReconnectPolicy {
            max_attempts: 1,
            initial_backoff: Duration::from_millis(10),
            ..pulselink_runtime::ReconnectPolicy::default()
        },
        ..CoyoteV3Config::default()
    };
    let session = client
        .connect_coyote_v3(&device(), config)
        .await
        .expect("connect");

    first.mark_disconnected();
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_millis(20)).await;
    for _ in 0..20 {
        tokio::task::yield_now().await;
    }
    assert_eq!(
        session.state().common.connection,
        ConnectionState::Disconnected
    );
}
