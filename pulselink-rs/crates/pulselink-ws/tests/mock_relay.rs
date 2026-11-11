#![cfg(feature = "mock")]
//! Mock-relay integration tests for the V4 socket.

use std::time::Duration;

use pulselink_ws::testing::MockRelay;
use pulselink_ws::{
    ActionType, AppEvent, AppMessage, Channel, ClearTarget, DeviceOperate, DeviceOperateResult,
    OperateValue, OperationEndReason, PulseFrameWire, RemoteDevice, SlotPatchEntry, Socket,
    SocketEvent, SocketOptions, WsError,
};
use serde_json::{Map, json};
use tokio::time::{sleep, timeout};

fn device(slot: &str, name: &str) -> RemoteDevice {
    RemoteDevice {
        id: None,
        slot_id: slot.to_owned(),
        name: name.to_owned(),
        device_type: "COYOTE_030".to_owned(),
        props: Map::new(),
        slot_state: Map::new(),
    }
}

fn set_zero(slot: &str, channel: Channel) -> DeviceOperate {
    DeviceOperate {
        slot_id: slot.to_owned(),
        action: ActionType::SetIntensity,
        channel: match channel {
            Channel::A => 0,
            Channel::B => 1,
        },
        priority: Some(1),
        duration_ms: None,
        value: OperateValue::Intensity(0),
        immediate: None,
        ver: None,
        seq: None,
    }
}

async fn connected() -> (MockRelay, Socket) {
    let relay = MockRelay::start().await.expect("relay");
    let socket = Socket::connect(relay.url()).await.expect("connect");
    (relay, socket)
}

async fn client(
    relay: &MockRelay,
    socket: &Socket,
    id: &str,
    devices: Vec<RemoteDevice>,
) -> pulselink_ws::AppClient {
    relay.attach_client(id, devices).await;
    timeout(Duration::from_secs(2), socket.wait_for_client())
        .await
        .expect("wait timeout")
        .expect("client")
}

async fn client_by_id(socket: &Socket, id: &str) -> pulselink_ws::AppClient {
    timeout(Duration::from_secs(2), async {
        loop {
            if let Some(client) = socket.client(id) {
                break client;
            }
            sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("client timeout")
}

async fn next_event(
    events: &mut tokio::sync::broadcast::Receiver<SocketEvent>,
    predicate: impl Fn(&SocketEvent) -> bool,
) -> SocketEvent {
    timeout(Duration::from_secs(2), async {
        loop {
            let event = events.recv().await.expect("event");
            if predicate(&event) {
                break event;
            }
        }
    })
    .await
    .expect("event timeout")
}

#[tokio::test]
async fn connects_attaches_and_requests_devices() {
    let (relay, socket) = connected().await;
    assert_eq!(socket.target_id(), "controller-1");
    assert!(matches!(
        socket.state(),
        pulselink_ws::SocketState::WaitingForPeer
    ));

    let client = client(&relay, &socket, "app-1", vec![device("slot-a", "A")]).await;
    assert_eq!(client.client_id(), "app-1");

    let devices = client.request_devices().await.expect("devices");
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].slot_id, "slot-a");

    let operation = client
        .submit_operate(set_zero("slot-a", Channel::A), Some(Duration::from_secs(2)))
        .await
        .expect("submit");
    let result = operation.completion().await.expect("completion");
    assert_eq!(result.action, ActionType::SetIntensity);

    client.clear_operate(ClearTarget::All).await.expect("clear");
    socket.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn emits_lifecycle_events() {
    let (relay, socket) = connected().await;
    let mut events = socket.subscribe();

    relay.attach_client("app-1", vec![]).await;
    let event = next_event(&mut events, |event| {
        matches!(event, SocketEvent::ClientAttached { .. })
    })
    .await;
    assert!(matches!(
        event,
        SocketEvent::ClientAttached { ref client_id } if client_id == "app-1"
    ));

    relay.heartbeat().await;
    next_event(&mut events, |event| matches!(event, SocketEvent::Heartbeat)).await;

    socket.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn caches_snapshot_patch_and_slots_patch() {
    let (relay, socket) = connected().await;
    let client = client(&relay, &socket, "app-1", vec![device("slot-a", "A")]).await;
    assert_eq!(client.devices().len(), 1);

    let mut updated = device("slot-a", "A2");
    updated.props = serde_json::from_value(json!({"intensityA": 10})).expect("object");
    relay
        .send_app_message(
            "app-1",
            AppMessage::Ev(AppEvent::DevicesPatch {
                added: vec![updated],
                removed: vec![],
            }),
        )
        .await;
    sleep(Duration::from_millis(20)).await;
    assert_eq!(client.devices()[0].name, "A2");
    assert_eq!(client.devices()[0].props["intensityA"], 10);

    relay
        .send_app_message(
            "app-1",
            AppMessage::Ev(AppEvent::SlotsPatch {
                slots: vec![SlotPatchEntry {
                    slot_id: "slot-a".to_owned(),
                    props: serde_json::from_value(json!({"intensityA": 20})).expect("object"),
                    slot_state: serde_json::from_value(json!({"hasDevice": true})).expect("object"),
                }],
            }),
        )
        .await;
    sleep(Duration::from_millis(20)).await;
    let devices = client.devices();
    assert_eq!(devices[0].props["intensityA"], 20);
    assert_eq!(devices[0].slot_state["hasDevice"], true);

    socket.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn multiple_apps_are_isolated() {
    let (relay, socket) = connected().await;
    relay
        .attach_client("app-1", vec![device("slot-a", "A")])
        .await;
    let first = client_by_id(&socket, "app-1").await;
    relay
        .attach_client("app-2", vec![device("slot-b", "B")])
        .await;
    let second = client_by_id(&socket, "app-2").await;
    assert_eq!(socket.clients().len(), 2);

    let first_devices = first.request_devices().await.expect("first devices");
    let second_devices = second.request_devices().await.expect("second devices");
    assert_eq!(first_devices[0].slot_id, "slot-a");
    assert_eq!(second_devices[0].slot_id, "slot-b");

    let first_op = first
        .submit_operate(set_zero("slot-a", Channel::A), Some(Duration::from_secs(2)))
        .await
        .expect("first op");
    let second_op = second
        .submit_operate(set_zero("slot-b", Channel::B), Some(Duration::from_secs(2)))
        .await
        .expect("second op");
    assert_eq!(
        first_op
            .completion()
            .await
            .expect("first")
            .slot_id
            .as_deref(),
        Some("slot-a")
    );
    assert_eq!(
        second_op
            .completion()
            .await
            .expect("second")
            .slot_id
            .as_deref(),
        Some("slot-b")
    );

    relay.detach_client("app-1").await;
    timeout(Duration::from_secs(2), async {
        while socket.clients().len() != 1 {
            sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("detach timeout");
    assert!(socket.client("app-1").is_none());

    socket.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn deferred_operation_is_cleared() {
    let (relay, socket) = connected().await;
    let client = client(&relay, &socket, "app-1", vec![device("slot-a", "A")]).await;
    relay.set_defer_operate(true).await;
    sleep(Duration::from_millis(10)).await;

    let operation = client
        .submit_operate(set_zero("slot-a", Channel::A), Some(Duration::from_secs(2)))
        .await
        .expect("submit");
    sleep(Duration::from_millis(20)).await;
    client
        .clear_operate(ClearTarget::Channel {
            slot_id: "slot-a".to_owned(),
            channel: Channel::A,
        })
        .await
        .expect("clear");
    let result = operation.completion().await.expect("completion");
    assert_eq!(result.reason, OperationEndReason::Cleared);

    socket.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn rpc_timeout_and_error_mapping() {
    let relay = MockRelay::start().await.expect("relay");
    let options = SocketOptions {
        url: relay.url().to_owned(),
        response_timeout: Duration::from_millis(50),
        ..SocketOptions::default()
    };
    let socket = Socket::connect_with(options).await.expect("connect");
    let client = client(&relay, &socket, "app-1", vec![]).await;

    relay.set_no_response(true).await;
    sleep(Duration::from_millis(10)).await;
    assert!(matches!(
        client.request_devices().await,
        Err(WsError::RpcTimeout { .. })
    ));

    relay.set_no_response(false).await;
    relay.set_rpc_error(Some("slot_not_found".to_owned())).await;
    sleep(Duration::from_millis(10)).await;
    assert!(matches!(
        client.request_devices().await,
        Err(WsError::DeviceNotFound(_))
    ));

    socket.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn detach_fails_pending_operations() {
    let (relay, socket) = connected().await;
    let client = client(&relay, &socket, "app-1", vec![device("slot-a", "A")]).await;
    relay.set_defer_operate(true).await;
    sleep(Duration::from_millis(10)).await;

    let operation = client
        .submit_operate(set_zero("slot-a", Channel::A), None)
        .await
        .expect("submit");
    relay.detach_client("app-1").await;
    assert!(matches!(
        operation.completion().await,
        Err(WsError::Disconnected)
    ));

    socket.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn close_code_is_reported() {
    let (relay, socket) = connected().await;
    let mut events = socket.subscribe();
    relay.close(4000, "controller disconnected").await;

    let event = next_event(&mut events, |event| {
        matches!(event, SocketEvent::Closed { .. })
    })
    .await;
    match event {
        SocketEvent::Closed { code, reason } => {
            assert_eq!(code, Some(4000));
            assert!(reason.contains("controller"));
        }
        _ => panic!("unexpected event"),
    }
    assert!(matches!(socket.state(), pulselink_ws::SocketState::Closed));
    assert!(socket.clients().is_empty());
}

#[tokio::test]
async fn server_ping_receives_pong() {
    let (relay, socket) = connected().await;
    let mut events = socket.subscribe();
    relay.pong(42).await;
    let event = next_event(&mut events, |event| {
        matches!(event, SocketEvent::Pong { .. })
    })
    .await;
    assert!(matches!(event, SocketEvent::Pong { ts: Some(42) }));
    socket.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn operation_result_round_trips() {
    let (relay, socket) = connected().await;
    let client = client(&relay, &socket, "app-1", vec![device("slot-a", "A")]).await;
    let operation = client
        .submit_operate(set_zero("slot-a", Channel::B), Some(Duration::from_secs(2)))
        .await
        .expect("submit");
    let result: DeviceOperateResult = operation.completion().await.expect("completion");
    assert_eq!(result.channel, Some(1));
    assert_eq!(result.reason, OperationEndReason::Completed);
    socket.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn supports_all_operation_types() {
    let (relay, socket) = connected().await;
    let client = client(&relay, &socket, "app-1", vec![device("slot-a", "A")]).await;
    let operations = vec![
        DeviceOperate {
            slot_id: "slot-a".to_owned(),
            action: ActionType::AppendPulseData,
            channel: 0,
            priority: Some(1),
            duration_ms: Some(1000),
            value: OperateValue::PulseFrames(vec![PulseFrameWire::Hex(
                "0A0A0A0A00000000".to_owned(),
            )]),
            immediate: None,
            ver: Some(3),
            seq: None,
        },
        DeviceOperate {
            slot_id: "slot-a".to_owned(),
            action: ActionType::AddIntensity,
            channel: 0,
            priority: Some(1),
            duration_ms: None,
            value: OperateValue::Intensity(5),
            immediate: None,
            ver: None,
            seq: None,
        },
        DeviceOperate {
            slot_id: "slot-a".to_owned(),
            action: ActionType::SetTempIntensity,
            channel: 1,
            priority: Some(1),
            duration_ms: Some(500),
            value: OperateValue::Intensity(20),
            immediate: None,
            ver: None,
            seq: None,
        },
        set_zero("slot-a", Channel::B),
    ];

    for operation in operations {
        let action = operation.action;
        let pending = client
            .submit_operate(operation, Some(Duration::from_secs(2)))
            .await
            .expect("submit");
        let result = pending.completion().await.expect("completion");
        assert_eq!(result.action, action);
    }

    socket.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn ping_interval_waits_for_server_pong() {
    let relay = MockRelay::start().await.expect("relay");
    let options = SocketOptions {
        url: relay.url().to_owned(),
        ping_interval: Some(Duration::from_millis(50)),
        response_timeout: Duration::from_millis(500),
        ..SocketOptions::default()
    };
    let socket = Socket::connect_with(options).await.expect("connect");
    let mut events = socket.subscribe();
    let event = next_event(&mut events, |event| {
        matches!(event, SocketEvent::Pong { ts: Some(_) })
    })
    .await;
    assert!(matches!(
        event,
        SocketEvent::Pong {
            ts: Some(123_456_789)
        }
    ));
    socket.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn idle_timeout_closes_the_socket() {
    let (relay, socket) = connected().await;
    let mut events = socket.subscribe();
    relay.idle_timeout().await;
    next_event(&mut events, |event| {
        matches!(event, SocketEvent::IdleTimeout)
    })
    .await;
    assert!(matches!(socket.state(), pulselink_ws::SocketState::Closed));
    assert!(socket.clients().is_empty());
}
