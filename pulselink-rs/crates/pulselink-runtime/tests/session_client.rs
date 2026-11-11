//! Session and client shutdown tests.

mod common;

use std::sync::Arc;
use std::time::Duration;

use pulselink_core::ConnectionState;
use pulselink_protocol::coyote::v3::gatt::{CHAR_BATTERY, CHAR_WRITE};
use pulselink_runtime::{Client, ClientConfig};
use pulselink_transport::TransportDeviceId;
use pulselink_transport::mock::{MockBackend, MockLink};

use common::{config, device};

#[tokio::test(start_paused = true)]
async fn shutdown_cancels_and_writes_zero_intensity() {
    let link = MockLink::new(TransportDeviceId::new("coyote-1"));
    link.set_read_data(CHAR_BATTERY, vec![80]);
    let backend = Arc::new(MockBackend::new(vec![device()]));
    backend.push_link(link.clone());
    let client = Client::with_config(backend, ClientConfig::default());
    let session = client
        .connect_coyote_v3(&device(), config())
        .await
        .expect("connect");

    let task = tokio::spawn(async move { session.shutdown().await });
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_millis(20)).await;
    let result = task.await.expect("join");
    assert!(result.is_ok());
    assert!(
        link.writes()
            .iter()
            .any(|write| write.characteristic == CHAR_WRITE && write.data.first() == Some(&0xB0))
    );
}

#[tokio::test(start_paused = true)]
async fn client_shutdown_closes_sessions() {
    let link = MockLink::new(TransportDeviceId::new("coyote-1"));
    link.set_read_data(CHAR_BATTERY, vec![80]);
    let backend = Arc::new(MockBackend::new(vec![device()]));
    backend.push_link(link);
    let client = Client::with_config(backend, ClientConfig::default());
    let session = client
        .connect_coyote_v3(&device(), config())
        .await
        .expect("connect");
    let state = session.clone();
    let shutdown = tokio::spawn(async move { client.shutdown().await });
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_millis(20)).await;
    shutdown.await.expect("join");
    assert_eq!(
        state.state().common.connection,
        ConnectionState::Disconnected
    );
}
