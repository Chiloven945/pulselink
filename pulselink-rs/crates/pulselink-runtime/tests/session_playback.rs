//! Pattern, scheduler, and stop tests.

mod common;

use std::sync::Arc;
use std::time::Duration;

use pulselink_pattern::{Pattern, StereoFrame};
use pulselink_protocol::coyote::v3::gatt::{CHAR_BATTERY, CHAR_NOTIFY, CHAR_WRITE};
use pulselink_runtime::{Client, ClientConfig, PlayEndReason, PlayOptions};
use pulselink_transport::TransportDeviceId;
use pulselink_transport::mock::{MockBackend, MockLink};

use common::{config, device};

#[tokio::test(start_paused = true)]
async fn stop_writes_zero_and_waits_for_b1() {
    let link = MockLink::new(TransportDeviceId::new("coyote-1"));
    link.set_read_data(CHAR_BATTERY, vec![80]);
    let backend = Arc::new(MockBackend::new(vec![device()]));
    backend.push_link(link.clone());
    let client = Client::with_config(backend, ClientConfig::default());
    let session = client
        .connect_coyote_v3(&device(), config())
        .await
        .expect("connect");

    let stopper = {
        let session = session.clone();
        tokio::spawn(async move { session.stop().await })
    };
    tokio::task::yield_now().await;
    let zero = link
        .writes()
        .iter()
        .rev()
        .find(|write| write.characteristic == CHAR_WRITE && write.data.first() == Some(&0xB0))
        .cloned()
        .expect("zero B0");
    assert_eq!(zero.data[2], 0);
    assert_eq!(zero.data[3], 0);
    let sequence = zero.data[1] >> 4;
    link.inject_notification(CHAR_NOTIFY, &[0xB1, sequence, 0, 0]);
    let outcome = stopper.await.expect("join").expect("stop");
    assert!(outcome.acknowledged);
}

#[tokio::test(start_paused = true)]
async fn pattern_finishes_and_play_handle_resolves() {
    let link = MockLink::new(TransportDeviceId::new("coyote-1"));
    link.set_read_data(CHAR_BATTERY, vec![80]);
    let backend = Arc::new(MockBackend::new(vec![device()]));
    backend.push_link(link.clone());
    let client = Client::with_config(backend, ClientConfig::default());
    let session = client
        .connect_coyote_v3(&device(), config())
        .await
        .expect("connect");

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
    let reason = waiter.await.expect("join").expect("wait");
    assert_eq!(reason, PlayEndReason::Completed);
    assert!(session.state().pattern.is_none());
}

#[tokio::test(start_paused = true)]
async fn stop_timeout_returns_unacknowledged_outcome() {
    let link = MockLink::new(TransportDeviceId::new("coyote-1"));
    link.set_read_data(CHAR_BATTERY, vec![80]);
    let backend = Arc::new(MockBackend::new(vec![device()]));
    backend.push_link(link.clone());
    let client = Client::with_config(backend, ClientConfig::default());
    let session = client
        .connect_coyote_v3(&device(), config())
        .await
        .expect("connect");
    let stopper = {
        let session = session.clone();
        tokio::spawn(async move { session.stop().await })
    };
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_millis(20)).await;
    let outcome = stopper.await.expect("join").expect("stop");
    assert!(!outcome.acknowledged);
}

#[tokio::test(start_paused = true)]
async fn scheduler_lag_pauses_pattern() {
    let link = MockLink::new(TransportDeviceId::new("coyote-1"));
    link.set_read_data(CHAR_BATTERY, vec![80]);
    let backend = Arc::new(MockBackend::new(vec![device()]));
    backend.push_link(link.clone());
    let client = Client::with_config(backend, ClientConfig::default());
    let mut config = config();
    config.lag_threshold = Duration::from_millis(50);
    let session = client
        .connect_coyote_v3(&device(), config)
        .await
        .expect("connect");
    let pattern = Pattern::new(
        vec![
            StereoFrame::silent(),
            StereoFrame::silent(),
            StereoFrame::silent(),
        ],
        true,
    )
    .expect("pattern");
    let handle = session
        .play(pattern, PlayOptions::default())
        .await
        .expect("play");
    let waiter = tokio::spawn(async move { handle.wait().await });
    tokio::time::advance(Duration::from_millis(100)).await;
    for _ in 0..10 {
        tokio::task::yield_now().await;
    }
    assert_eq!(
        waiter.await.expect("join").expect("wait"),
        PlayEndReason::SchedulerLag
    );
    assert!(session.state().pattern.is_none());
}

#[tokio::test(start_paused = true)]
async fn replacing_a_pattern_finishes_the_old_handle() {
    let link = MockLink::new(TransportDeviceId::new("coyote-1"));
    link.set_read_data(CHAR_BATTERY, vec![80]);
    let backend = Arc::new(MockBackend::new(vec![device()]));
    backend.push_link(link);
    let client = Client::with_config(backend, ClientConfig::default());
    let session = client
        .connect_coyote_v3(&device(), config())
        .await
        .expect("connect");
    let first = session
        .play(
            Pattern::new(vec![StereoFrame::silent()], true).expect("pattern"),
            PlayOptions::default(),
        )
        .await
        .expect("first");
    let second = session
        .play(
            Pattern::new(vec![StereoFrame::silent()], true).expect("pattern"),
            PlayOptions { replace: true },
        )
        .await
        .expect("second");
    assert_eq!(
        first.wait().await.expect("first wait"),
        PlayEndReason::Replaced
    );
    second.stop().await.expect("stop second");
}
