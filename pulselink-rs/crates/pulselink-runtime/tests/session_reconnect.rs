//! Reconnect tests.

mod common;

use std::sync::Arc;
use std::time::Duration;

use pulselink_core::ConnectionState;
use pulselink_pattern::{Pattern, StereoFrame};
use pulselink_protocol::coyote::v3::gatt::{CHAR_BATTERY, CHAR_WRITE};
use pulselink_runtime::{Client, ClientConfig, PlayOptions};
use pulselink_transport::TransportDeviceId;
use pulselink_transport::mock::{MockBackend, MockLink};

use common::{config, device};

#[tokio::test(start_paused = true)]
async fn disconnect_enters_degraded_and_reconnects_without_restoring_pattern() {
    let first = MockLink::new(TransportDeviceId::new("coyote-1"));
    first.set_read_data(CHAR_BATTERY, vec![70]);
    let backend = Arc::new(MockBackend::new(vec![device()]));
    backend.push_link(first.clone());
    let client = Client::with_config(backend.clone(), ClientConfig::default());
    let session = client
        .connect_coyote_v3(&device(), config())
        .await
        .expect("connect");

    let second = MockLink::new(TransportDeviceId::new("coyote-1"));
    second.set_read_data(CHAR_BATTERY, vec![69]);
    backend.push_link(second.clone());

    let _playback = session
        .play(
            Pattern::new(vec![StereoFrame::silent()], true).expect("pattern"),
            PlayOptions::default(),
        )
        .await
        .expect("play");
    assert!(session.state().pattern.is_some());
    first.mark_disconnected();
    tokio::task::yield_now().await;
    assert!(matches!(
        session.state().common.connection,
        ConnectionState::Degraded | ConnectionState::Reconnecting
    ));

    tokio::time::advance(Duration::from_millis(500)).await;
    for _ in 0..10 {
        tokio::task::yield_now().await;
    }
    tokio::time::advance(Duration::from_millis(100)).await;
    for _ in 0..10 {
        tokio::task::yield_now().await;
    }

    assert_eq!(session.state().common.connection, ConnectionState::Ready);
    assert!(session.state().pattern.is_none());
    assert!(
        second
            .writes()
            .iter()
            .any(|write| write.characteristic == CHAR_WRITE && write.data.first() == Some(&0xBF))
    );
}
