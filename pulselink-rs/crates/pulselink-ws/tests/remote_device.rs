#![cfg(feature = "mock")]
//! Remote device handle and operation tests.

use std::time::Duration;

use pulselink_ws::testing::MockRelay;
use pulselink_ws::{
    ActionType, AppMessage, Channel, DeviceOperate, OperateValue, OperationEndReason,
    OperationOptions, PulseFrame, RelayFrame, RemoteDevice, RpcRequest, Socket, WsError,
};
use serde_json::{Map, json};

fn device() -> RemoteDevice {
    RemoteDevice {
        id: Some(0),
        slot_id: "slot-a".to_owned(),
        name: "Device A".to_owned(),
        device_type: "COYOTE_030".to_owned(),
        props: serde_json::from_value(json!({
            "intensityA": 10,
            "intensityB": "20",
            "power": 88
        }))
        .expect("props"),
        slot_state: serde_json::from_value(json!({
            "hasDevice": true,
            "markLight": "green"
        }))
        .expect("slot state"),
    }
}

async fn connected() -> (
    MockRelay,
    Socket,
    pulselink_ws::AppClient,
    pulselink_ws::RemoteDeviceHandle,
) {
    let relay = MockRelay::start().await.expect("relay");
    let socket = Socket::connect(relay.url()).await.expect("connect");
    relay.attach_client("app-1", vec![device()]).await;
    let client = socket.wait_for_client().await.expect("client");
    let devices = client.request_devices().await.expect("devices");
    assert_eq!(devices.len(), 1);
    let handle = client.device("slot-a").expect("device handle");
    (relay, socket, client, handle)
}

async fn next_request(relay: &MockRelay) -> RpcRequest {
    loop {
        let frame = relay.recv().await.expect("frame");
        if let RelayFrame::Message { data, .. } = frame
            && let Ok(AppMessage::Req(request)) = serde_json::from_value(data)
            && request.m == "device.op"
        {
            return request;
        }
    }
}

fn operate(request: &RpcRequest) -> DeviceOperate {
    serde_json::from_value(request.data.clone().expect("operation data")).expect("operate")
}

#[tokio::test]
async fn snapshot_accessors_and_live_updates() {
    let (relay, socket, client, handle) = connected().await;
    let snapshot = handle.snapshot();
    assert_eq!(snapshot.slot_id, "slot-a");
    assert_eq!(snapshot.name, "Device A");
    assert_eq!(snapshot.device_type, "COYOTE_030");
    assert_eq!(snapshot.intensity_a(), Some(10));
    assert_eq!(snapshot.intensity_b(), Some(20));
    assert_eq!(snapshot.battery(), Some(88));
    assert_eq!(snapshot.has_device(), Some(true));
    assert_eq!(snapshot.mark_light(), Some("green"));
    assert_eq!(handle.name(), "Device A");

    let mut events = client.subscribe();
    relay
        .send_app_message(
            "app-1",
            AppMessage::Ev(pulselink_ws::AppEvent::SlotsPatch {
                slots: vec![pulselink_ws::SlotPatchEntry {
                    slot_id: "slot-a".to_owned(),
                    props: serde_json::from_value(json!({"intensityA": 42})).expect("props"),
                    slot_state: Map::new(),
                }],
            }),
        )
        .await;
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if let pulselink_ws::WsEvent::SlotsPatch { .. } = events.recv().await.expect("event") {
                break;
            }
        }
    })
    .await
    .expect("patch event");
    assert_eq!(handle.snapshot().intensity_a(), Some(42));
    assert_eq!(
        client
            .device("slot-a")
            .expect("still present")
            .snapshot()
            .intensity_a(),
        Some(42)
    );
    socket.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn intensity_operations_map_to_device_op() {
    let (relay, socket, _client, handle) = connected().await;

    let options = OperationOptions::default()
        .with_priority(2)
        .with_immediate(true);
    let pending = handle
        .add_intensity(Channel::A, 5, options)
        .await
        .expect("add");
    let request = next_request(&relay).await;
    let op = operate(&request);
    assert_eq!(op.action, ActionType::AddIntensity);
    assert_eq!(op.channel, 0);
    assert_eq!(op.priority, Some(2));
    assert_eq!(op.immediate, Some(true));
    assert_eq!(op.value, OperateValue::Intensity(5));
    let result = pending.completion().await.expect("completion");
    assert_eq!(result.action, ActionType::AddIntensity);
    assert_eq!(result.reason, OperationEndReason::Completed);
    assert_eq!(result.channel, Some(Channel::A));

    let pending = handle
        .reduce_intensity(Channel::B, 7, OperationOptions::default())
        .await
        .expect("reduce");
    let op = operate(&next_request(&relay).await);
    assert_eq!(op.value, OperateValue::Intensity(-7));
    pending.completion().await.expect("completion");

    let pending = handle
        .reset_intensity(Channel::B, OperationOptions::default())
        .await
        .expect("reset");
    let op = operate(&next_request(&relay).await);
    assert_eq!(op.action, ActionType::SetIntensity);
    assert_eq!(op.value, OperateValue::Intensity(0));
    pending.completion().await.expect("completion");

    socket.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn temp_intensity_and_pulse_frames_are_encoded() {
    let (relay, socket, _client, handle) = connected().await;

    let pending = handle
        .set_temp_intensity(
            Channel::A,
            pulselink_ws::Intensity::new(20).expect("intensity"),
            Duration::from_millis(350),
            OperationOptions::default(),
        )
        .await
        .expect("temp");
    let op = operate(&next_request(&relay).await);
    assert_eq!(op.action, ActionType::SetTempIntensity);
    assert_eq!(op.duration_ms, Some(350));
    assert_eq!(op.value, OperateValue::Intensity(20));
    pending.completion().await.expect("completion");

    let frame = PulseFrame::new([1, 2, 3, 4, 5, 6, 7, 8]);
    let pending = handle
        .send_pulse(
            Channel::B,
            Duration::from_millis(1000),
            vec![frame],
            OperationOptions::default(),
        )
        .await
        .expect("pulse");
    let op = operate(&next_request(&relay).await);
    assert_eq!(op.action, ActionType::AppendPulseData);
    assert_eq!(op.channel, 1);
    assert_eq!(op.duration_ms, Some(1000));
    assert_eq!(op.ver, Some(3));
    assert_eq!(
        op.value,
        OperateValue::PulseFrames(vec![pulselink_ws::PulseFrameWire::Numbers(vec![
            1, 2, 3, 4, 5, 6, 7, 8
        ])])
    );
    pending.completion().await.expect("completion");
    socket.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn invalid_values_are_rejected_before_send() {
    let (_relay, socket, _client, handle) = connected().await;
    assert!(matches!(
        handle
            .add_intensity(Channel::A, 201, OperationOptions::default())
            .await,
        Err(WsError::InvalidParams(_))
    ));
    let empty_pulse = handle
        .send_pulse(
            Channel::A,
            Duration::from_millis(100),
            Vec::new(),
            OperationOptions::default(),
        )
        .await;
    assert!(matches!(empty_pulse, Err(WsError::InvalidParams(_))));
    socket.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn operation_handle_reports_cleared_reason() {
    let (relay, socket, _client, handle) = connected().await;
    relay.set_defer_operate(true).await;
    tokio::time::sleep(Duration::from_millis(20)).await;

    let pending = handle
        .reset_intensity(Channel::A, OperationOptions::default())
        .await
        .expect("reset");
    tokio::time::sleep(Duration::from_millis(20)).await;
    socket
        .wait_for_client()
        .await
        .expect("client")
        .clear_operate(pulselink_ws::ClearTarget::Device {
            slot_id: "slot-a".to_owned(),
        })
        .await
        .expect("clear");
    let result = pending.completion().await.expect("completion");
    assert_eq!(result.reason, OperationEndReason::Cleared);
    socket.shutdown().await.expect("shutdown");
}
