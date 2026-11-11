//! End-to-end V4 remote-device scenarios with the mock relay.

use std::time::Duration;

use pulselink_ws::testing::MockRelay;
use pulselink_ws::{ActionType, Channel, OperationOptions, RemoteDevice, Socket};
use serde_json::{Map, json};

fn device() -> RemoteDevice {
    RemoteDevice {
        id: Some(0),
        slot_id: "slot-a".to_owned(),
        name: "Device A".to_owned(),
        device_type: "COYOTE_030".to_owned(),
        props: serde_json::from_value(json!({"power": 90, "intensityA": 5})).expect("props"),
        slot_state: Map::new(),
    }
}

#[tokio::test]
async fn remote_device_handle_operations() {
    let relay = MockRelay::start().await.expect("relay");
    let socket = Socket::connect(relay.url()).await.expect("connect");
    relay.attach_client("app-1", vec![device()]).await;
    let client = socket.wait_for_client().await.expect("client");
    let devices = client.request_devices().await.expect("devices");
    let handle = client.device(&devices[0].slot_id).expect("device handle");
    assert_eq!(handle.snapshot().battery(), Some(90));
    let result = handle
        .add_intensity(Channel::A, 5, OperationOptions::default())
        .await
        .expect("add")
        .completion()
        .await
        .expect("completion");
    assert_eq!(result.action, ActionType::AddIntensity);
    socket.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn handles_update_after_slots_patch() {
    let relay = MockRelay::start().await.expect("relay");
    let socket = Socket::connect(relay.url()).await.expect("connect");
    relay.attach_client("app-1", vec![device()]).await;
    let client = socket.wait_for_client().await.expect("client");
    let devices = client.request_devices().await.expect("devices");
    let handle = client.device(&devices[0].slot_id).expect("device");
    let mut events = client.subscribe();
    relay
        .send_app_message(
            "app-1",
            pulselink_ws::AppMessage::Ev(pulselink_ws::AppEvent::SlotsPatch {
                slots: vec![pulselink_ws::SlotPatchEntry {
                    slot_id: "slot-a".to_owned(),
                    props: serde_json::from_value(json!({"intensityA": 25})).expect("props"),
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
    assert_eq!(handle.snapshot().intensity_a(), Some(25));
    socket.shutdown().await.expect("shutdown");
}
