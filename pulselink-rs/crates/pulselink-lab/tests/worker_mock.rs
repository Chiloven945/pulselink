//! Worker integration tests with the in-memory transport.

use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use pulselink::protocol::coyote::v2::SERVICE_PWM;
use pulselink::protocol::coyote::v3::gatt::SERVICE_CONTROL;
use pulselink::protocol::socket::v4::{AppMessage, RelayFrame, RemoteDevice, RpcResponse};
use pulselink::transport::mock::{MockBackend, MockLink};
use pulselink::transport::{DiscoveredDevice, TransportDeviceId};
use pulselink::{Channel, DeviceKind, PulseLink};
use pulselink_lab::command::Command;
use pulselink_lab::model::Update;
use pulselink_lab::worker::LabWorker;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

fn device() -> DiscoveredDevice {
    DiscoveredDevice {
        id: TransportDeviceId::new("v2"),
        name: Some("D-LAB ESTIM01".to_owned()),
        rssi: Some(-40),
        service_uuids: vec![SERVICE_PWM],
        manufacturer_data: Vec::new(),
    }
}

fn unnamed_coyote() -> DiscoveredDevice {
    DiscoveredDevice {
        id: TransportDeviceId::new("aa:bb:cc:dd:ee:ff"),
        name: None,
        rssi: Some(-50),
        service_uuids: vec![SERVICE_CONTROL],
        manufacturer_data: Vec::new(),
    }
}

fn recv_until(worker: &LabWorker, predicate: impl Fn(&Update) -> bool) -> Update {
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        let update = worker
            .recv_timeout(remaining)
            .unwrap_or_else(|| panic!("worker did not produce expected update"));
        if predicate(&update) {
            return update;
        }
    }
}

#[tokio::test]
async fn scans_connects_and_sets_v2_intensity() {
    let device = device();
    let link = MockLink::new(device.id.clone());
    let backend = MockBackend::new(vec![device.clone()]);
    backend.push_link(link.clone());
    let worker = LabWorker::spawn_with(PulseLink::with_backend(Arc::new(backend)));

    worker.send(Command::Scan(Some(DeviceKind::CoyoteV2)));
    recv_until(&worker, |update| matches!(update, Update::Devices(_)));

    worker.send(Command::Connect(0));
    recv_until(&worker, |update| matches!(update, Update::Connected { .. }));

    worker.send(Command::SetOutput(true));
    worker.send(Command::V2Intensity(Channel::A, 70));
    recv_until(
        &worker,
        |update| matches!(update, Update::Log { message, .. } if message.contains("V2 intensity")),
    );

    let writes = link.writes();
    assert!(writes.iter().any(|write| write.data == [0, 0x30, 0x02]));

    worker.send(Command::StopAll);
    recv_until(
        &worker,
        |update| matches!(update, Update::Log { message, .. } if message.contains("emergency stop")),
    );
    worker.shutdown();
}

#[tokio::test]
async fn output_gate_blocks_nonzero_intensity() {
    let device = device();
    let link = MockLink::new(device.id.clone());
    let backend = MockBackend::new(vec![device.clone()]);
    backend.push_link(link.clone());
    let worker = LabWorker::spawn_with(PulseLink::with_backend(Arc::new(backend)));

    worker.send(Command::Scan(Some(DeviceKind::CoyoteV2)));
    recv_until(&worker, |update| matches!(update, Update::Devices(_)));
    worker.send(Command::Connect(0));
    recv_until(&worker, |update| matches!(update, Update::Connected { .. }));

    worker.send(Command::V2Intensity(Channel::A, 70));
    recv_until(
        &worker,
        |update| matches!(update, Update::Error(message) if message.contains("output is disabled")),
    );
    assert!(
        !link
            .writes()
            .iter()
            .any(|write| write.data == [0, 0x30, 0x02])
    );
    worker.shutdown();
}

#[tokio::test]
async fn local_relay_starts_and_stops() {
    let backend = MockBackend::new(Vec::new());
    let worker = LabWorker::spawn_with(PulseLink::with_backend(Arc::new(backend)));
    worker.send(Command::RelayStart(Some(
        "127.0.0.1:0".parse().expect("loopback"),
    )));
    recv_until(
        &worker,
        |update| matches!(update, Update::Log { message, .. } if message.contains("local relay listening")),
    );
    worker.send(Command::RelayStop);
    recv_until(
        &worker,
        |update| matches!(update, Update::Log { message, .. } if message.contains("local relay stopped")),
    );
    worker.shutdown();
}

fn log_message(update: &Update) -> Option<&str> {
    match update {
        Update::Log { message, .. } => Some(message),
        _ => None,
    }
}

#[tokio::test]
async fn local_relay_pairs_with_a_real_app() {
    let backend = MockBackend::new(Vec::new());
    let worker = LabWorker::spawn_with(PulseLink::with_backend(Arc::new(backend)));
    worker.send(Command::WsLocal(Some(
        "127.0.0.1:0".parse().expect("loopback"),
    )));
    let update = recv_until(&worker, |update| {
        log_message(update).is_some_and(|message| message.contains("V4 target id:"))
    });
    let message = log_message(&update).expect("log");
    let target = message
        .split("V4 target id: ")
        .nth(1)
        .expect("target id")
        .to_owned();
    let update = recv_until(&worker, |update| {
        log_message(update).is_some_and(|message| message.contains("pairing URL:"))
    });
    let message = log_message(&update).expect("log");
    let pairing = message
        .split("pairing URL: ")
        .nth(1)
        .expect("pairing URL")
        .to_owned();
    assert!(pairing.ends_with(&target));

    let (mut app, _) = connect_async(pairing).await.expect("app connect");
    loop {
        let message = app.next().await.expect("app frame").expect("app message");
        let Message::Text(text) = message else {
            continue;
        };
        let frame: RelayFrame = serde_json::from_str(&text).expect("relay frame");
        if matches!(frame, RelayFrame::ControllerAttached { .. }) {
            break;
        }
    }

    let req_id = loop {
        let message = app.next().await.expect("app frame").expect("app message");
        let Message::Text(text) = message else {
            continue;
        };
        let frame: RelayFrame = serde_json::from_str(&text).expect("relay frame");
        let RelayFrame::Message { data, .. } = frame else {
            continue;
        };
        let Ok(AppMessage::Req(request)) = serde_json::from_value::<AppMessage>(data) else {
            continue;
        };
        if request.m == "devices.get" {
            break request.req_id;
        }
    };

    let devices = vec![RemoteDevice {
        id: Some(0),
        slot_id: "real-app-coyote".to_owned(),
        name: "Real App Coyote".to_owned(),
        device_type: "COYOTE_030".to_owned(),
        props: serde_json::Map::default(),
        slot_state: serde_json::Map::default(),
    }];
    let data = AppMessage::Resp(RpcResponse {
        req_id,
        result: Some(serde_json::json!({ "devices": devices })),
        error: None,
    });
    app.send(Message::Text(
        serde_json::to_string(&RelayFrame::Message {
            client_id: None,
            data: serde_json::to_value(data).expect("message"),
        })
        .expect("frame")
        .into(),
    ))
    .await
    .expect("app response");

    recv_until(
        &worker,
        |update| matches!(update, Update::WsState { devices, .. } if !devices.is_empty()),
    );

    worker.send(Command::RelayStop);
    worker.shutdown();
}

#[tokio::test]
async fn v3_scan_falls_back_for_unnamed_device() {
    let backend = MockBackend::new(vec![unnamed_coyote()]);
    let worker = LabWorker::spawn_with(PulseLink::with_backend(Arc::new(backend)));
    worker.send(Command::Scan(Some(DeviceKind::CoyoteV3)));
    let update = recv_until(&worker, |update| matches!(update, Update::Devices(_)));
    let Update::Devices(devices) = update else {
        unreachable!()
    };
    assert_eq!(devices.len(), 1);
    assert!(devices[0].name.is_none());
    worker.shutdown();
}
