//! Intensity ACK and queue tests.

mod common;

use std::sync::Arc;
use std::time::Duration;

use pulselink_core::{Channel, Intensity};
use pulselink_protocol::coyote::v3::gatt::{CHAR_BATTERY, CHAR_NOTIFY, CHAR_WRITE};
use pulselink_runtime::{Client, ClientConfig, CoyoteV3Event};
use pulselink_transport::TransportDeviceId;
use pulselink_transport::mock::{MockBackend, MockLink};

use common::{config, device};

#[tokio::test(start_paused = true)]
async fn matching_b1_completes_intensity_and_updates_observed_state() {
    let link = MockLink::new(TransportDeviceId::new("coyote-1"));
    link.set_read_data(CHAR_BATTERY, vec![80]);
    let backend = Arc::new(MockBackend::new(vec![device()]));
    backend.push_link(link.clone());
    let client = Client::with_config(backend, ClientConfig::default());
    let session = client
        .connect_coyote_v3(&device(), config())
        .await
        .expect("connect");

    let setter = {
        let session = session.clone();
        tokio::spawn(async move {
            session
                .set_intensity(Channel::A, Intensity::new(25).expect("valid"))
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
        .find(|write| write.characteristic == CHAR_WRITE && write.data.first() == Some(&0xB0))
        .map(|write| write.data[1] >> 4)
        .expect("B0 write");
    assert_ne!(sequence, 0);
    link.inject_notification(CHAR_NOTIFY, &[0xB1, sequence, 25, 0]);

    let change = setter.await.expect("task").expect("intensity");
    assert_eq!(change.channel, Channel::A);
    assert_eq!(change.applied.get(), 25);
    assert_eq!(
        session.state().common.channels[Channel::A]
            .intensity
            .value()
            .map(|value| value.get()),
        Some(25)
    );
}

#[tokio::test(start_paused = true)]
async fn ack_timeout_marks_intensity_unknown() {
    let link = MockLink::new(TransportDeviceId::new("coyote-1"));
    link.set_read_data(CHAR_BATTERY, vec![80]);
    let backend = Arc::new(MockBackend::new(vec![device()]));
    backend.push_link(link.clone());
    let client = Client::with_config(backend, ClientConfig::default());
    let session = client
        .connect_coyote_v3(&device(), config())
        .await
        .expect("connect");
    let mut events = session.subscribe();

    let setter = {
        let session = session.clone();
        tokio::spawn(async move {
            session
                .set_intensity(Channel::A, Intensity::new(20).expect("valid"))
                .await
        })
    };
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_millis(100)).await;
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_millis(10)).await;
    let result = setter.await.expect("join");
    assert!(matches!(
        result,
        Err(pulselink_runtime::RuntimeError::AckTimeout(_))
    ));
    assert!(
        session.state().common.channels[Channel::A]
            .intensity
            .is_unknown()
    );
    let event = tokio::time::timeout(Duration::from_millis(10), events.recv())
        .await
        .expect("event")
        .expect("event");
    assert!(matches!(event, CoyoteV3Event::AckTimeout { .. }));
}

#[tokio::test(start_paused = true)]
async fn queued_intensity_requests_are_serialized() {
    let link = MockLink::new(TransportDeviceId::new("coyote-1"));
    link.set_read_data(CHAR_BATTERY, vec![80]);
    let backend = Arc::new(MockBackend::new(vec![device()]));
    backend.push_link(link.clone());
    let client = Client::with_config(backend, ClientConfig::default());
    let session = client
        .connect_coyote_v3(&device(), config())
        .await
        .expect("connect");

    let first = {
        let session = session.clone();
        tokio::spawn(async move {
            session
                .set_intensity(Channel::A, Intensity::new(10).expect("valid"))
                .await
        })
    };
    let second = {
        let session = session.clone();
        tokio::spawn(async move {
            session
                .set_intensity(Channel::A, Intensity::new(20).expect("valid"))
                .await
        })
    };
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_millis(100)).await;
    tokio::task::yield_now().await;

    let first_sequence = link
        .writes()
        .iter()
        .rev()
        .find(|write| write.characteristic == CHAR_WRITE && write.data.first() == Some(&0xB0))
        .map(|write| write.data[1] >> 4)
        .expect("first B0");
    link.inject_notification(CHAR_NOTIFY, &[0xB1, first_sequence, 10, 0]);
    first.await.expect("join").expect("first");
    tokio::time::advance(Duration::from_millis(100)).await;
    tokio::task::yield_now().await;

    let second_sequence = link
        .writes()
        .iter()
        .rev()
        .find(|write| write.characteristic == CHAR_WRITE && write.data.first() == Some(&0xB0))
        .map(|write| write.data[1] >> 4)
        .expect("second B0");
    link.inject_notification(CHAR_NOTIFY, &[0xB1, second_sequence, 20, 0]);
    second.await.expect("join").expect("second");
}

#[tokio::test(start_paused = true)]
async fn unsolicited_b1_updates_state_without_completing_pending() {
    let link = MockLink::new(TransportDeviceId::new("coyote-1"));
    link.set_read_data(CHAR_BATTERY, vec![80]);
    let backend = Arc::new(MockBackend::new(vec![device()]));
    backend.push_link(link.clone());
    let client = Client::with_config(backend, ClientConfig::default());
    let session = client
        .connect_coyote_v3(&device(), config())
        .await
        .expect("connect");
    let setter = {
        let session = session.clone();
        tokio::spawn(async move {
            session
                .set_intensity(Channel::A, Intensity::new(40).expect("valid"))
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
    link.inject_notification(CHAR_NOTIFY, &[0xB1, 0, 33, 0]);
    tokio::task::yield_now().await;
    assert!(!setter.is_finished());
    assert_eq!(
        session.state().common.channels[Channel::A]
            .intensity
            .value()
            .map(|value| value.get()),
        Some(33)
    );
    link.inject_notification(CHAR_NOTIFY, &[0xB1, sequence, 40, 0]);
    let change = setter.await.expect("join").expect("change");
    assert_eq!(change.applied.get(), 40);
}

#[tokio::test(start_paused = true)]
async fn mismatched_b1_emits_diagnostic_without_completing() {
    let link = MockLink::new(TransportDeviceId::new("coyote-1"));
    link.set_read_data(CHAR_BATTERY, vec![80]);
    let backend = Arc::new(MockBackend::new(vec![device()]));
    backend.push_link(link.clone());
    let client = Client::with_config(backend, ClientConfig::default());
    let session = client
        .connect_coyote_v3(&device(), config())
        .await
        .expect("connect");
    let mut events = session.subscribe();
    let setter = {
        let session = session.clone();
        tokio::spawn(async move {
            session
                .set_intensity(Channel::A, Intensity::new(40).expect("valid"))
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
    let wrong = if sequence == 15 { 1 } else { sequence + 1 };
    link.inject_notification(CHAR_NOTIFY, &[0xB1, wrong, 33, 0]);
    let mut mismatch = false;
    for _ in 0..10 {
        if let Ok(event) = tokio::time::timeout(Duration::from_millis(5), events.recv()).await
            && matches!(event, Ok(CoyoteV3Event::B1Mismatch { .. }))
        {
            mismatch = true;
            break;
        }
    }
    assert!(mismatch);
    assert!(!setter.is_finished());
    link.inject_notification(CHAR_NOTIFY, &[0xB1, sequence, 40, 0]);
    setter.await.expect("join").expect("change");
}
