//! End-to-end runtime scenarios with the mock GATT backend.

use std::sync::Arc;
use std::time::Duration;

use pulselink_core::{Channel, ChannelMap, ConnectionState, Intensity};
use pulselink_pattern::{Pattern, StereoFrame};
use pulselink_protocol::coyote::v3::gatt::{CHAR_BATTERY, CHAR_NOTIFY, CHAR_WRITE};
use pulselink_runtime::{Client, ClientConfig, CoyoteV3Config, PlayEndReason, PlayOptions};
use pulselink_transport::mock::{MockBackend, MockLink};
use pulselink_transport::{DiscoveredDevice, TransportDeviceId};

fn device() -> DiscoveredDevice {
    DiscoveredDevice {
        id: TransportDeviceId::new("coyote-1"),
        name: Some("47L121000".to_owned()),
        rssi: Some(-40),
        service_uuids: Vec::new(),
        manufacturer_data: Vec::new(),
    }
}

fn config() -> CoyoteV3Config {
    CoyoteV3Config {
        ack_timeout: Duration::from_millis(10),
        ..CoyoteV3Config::default()
    }
}

async fn session() -> (MockLink, pulselink_runtime::CoyoteV3Session) {
    let link = MockLink::new(TransportDeviceId::new("coyote-1"));
    link.set_read_data(CHAR_BATTERY, vec![90]);
    let backend = MockBackend::new(vec![device()]);
    backend.push_link(link.clone());
    let client = Client::with_config(Arc::new(backend), ClientConfig::default());
    let session = client
        .connect_coyote_v3(&device(), config())
        .await
        .expect("connect");
    (link, session)
}

#[tokio::test(start_paused = true)]
async fn initialization_reaches_ready_and_writes_bf() {
    let (link, session) = session().await;
    assert_eq!(session.state().common.connection, ConnectionState::Ready);
    assert!(
        link.writes()
            .iter()
            .any(|write| write.characteristic == CHAR_WRITE && write.data[0] == 0xBF)
    );
}

#[tokio::test(start_paused = true)]
async fn intensity_ack_and_state_update() {
    let (link, session) = session().await;
    let task = {
        let session = session.clone();
        tokio::spawn(async move {
            session
                .set_intensity(Channel::A, Intensity::new(30).expect("valid"))
                .await
        })
    };
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_millis(100)).await;
    tokio::task::yield_now().await;
    let sequence = link
        .writes()
        .iter()
        .rev()
        .find(|write| write.characteristic == CHAR_WRITE && write.data[0] == 0xB0)
        .map(|write| write.data[1] >> 4)
        .expect("B0");
    link.inject_notification(CHAR_NOTIFY, &[0xB1, sequence, 30, 0]);
    let change = task.await.expect("join").expect("change");
    assert_eq!(change.applied.get(), 30);
}

#[tokio::test(start_paused = true)]
async fn pattern_playback_and_completion() {
    let (_link, session) = session().await;
    let pattern =
        Pattern::new(vec![StereoFrame::silent(), StereoFrame::silent()], false).expect("pattern");
    let handle = session
        .play(pattern, PlayOptions::default())
        .await
        .expect("play");
    let waiter = tokio::spawn(async move { handle.wait().await });
    tokio::time::advance(Duration::from_millis(300)).await;
    for _ in 0..10 {
        tokio::task::yield_now().await;
    }
    assert_eq!(
        waiter.await.expect("join").expect("wait"),
        PlayEndReason::Completed
    );
}

#[tokio::test(start_paused = true)]
async fn stop_returns_acknowledged_outcome() {
    let (link, session) = session().await;
    let stopper = {
        let session = session.clone();
        tokio::spawn(async move { session.stop().await })
    };
    tokio::task::yield_now().await;
    let zero = link
        .writes()
        .iter()
        .rev()
        .find(|write| write.characteristic == CHAR_WRITE && write.data[0] == 0xB0)
        .cloned()
        .expect("zero B0");
    let sequence = zero.data[1] >> 4;
    link.inject_notification(CHAR_NOTIFY, &[0xB1, sequence, 0, 0]);
    assert!(stopper.await.expect("join").expect("stop").acknowledged);
}

#[tokio::test(start_paused = true)]
async fn soft_limit_can_be_changed() {
    let (link, session) = session().await;
    session
        .set_soft_limit(Channel::A, Intensity::new(10).expect("valid"))
        .await
        .expect("soft limit");
    assert_eq!(
        session
            .state()
            .settings
            .value()
            .map(|settings| settings.soft_limit.a.get()),
        Some(10)
    );
    assert!(
        link.writes()
            .iter()
            .rev()
            .any(|write| write.characteristic == CHAR_WRITE && write.data[0] == 0xBF)
    );
}

#[tokio::test(start_paused = true)]
async fn safety_policy_blocks_excessive_intensity() {
    let link = MockLink::new(TransportDeviceId::new("coyote-1"));
    link.set_read_data(CHAR_BATTERY, vec![90]);
    let backend = MockBackend::new(vec![device()]);
    backend.push_link(link);
    let client = Client::with_config(Arc::new(backend), ClientConfig::default());
    let mut config = config();
    config.safety.max_intensity =
        ChannelMap::new(Intensity::new(5).expect("valid"), Intensity::MAX);
    let session = client
        .connect_coyote_v3(&device(), config)
        .await
        .expect("connect");
    assert!(
        session
            .set_intensity(Channel::A, Intensity::new(6).expect("valid"))
            .await
            .is_err()
    );
}
