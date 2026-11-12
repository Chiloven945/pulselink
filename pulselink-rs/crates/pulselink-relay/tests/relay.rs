//! End-to-end tests with real controller and app WebSocket clients.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use pulselink_protocol::socket::v4::RelayFrame;
use pulselink_relay::{LocalRelay, RelayConfig};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

type Client = WebSocketStream<MaybeTlsStream<TcpStream>>;

fn config() -> RelayConfig {
    RelayConfig {
        bind: "127.0.0.1:0".parse().expect("loopback address"),
        heartbeat_interval: Duration::from_secs(3600),
        idle_timeout: None,
        ..RelayConfig::default()
    }
}

async fn connect(url: &str) -> Client {
    let (client, _) = connect_async(url).await.expect("connect websocket");
    client
}

async fn send(client: &mut Client, frame: &RelayFrame) {
    let text = serde_json::to_string(frame).expect("serialize frame");
    client
        .send(Message::Text(text.into()))
        .await
        .expect("send frame");
}

async fn next_frame(client: &mut Client) -> RelayFrame {
    timeout(Duration::from_secs(2), async {
        loop {
            let message = client.next().await.expect("socket open").expect("message");
            match message {
                Message::Text(text) => return serde_json::from_str(&text).expect("valid frame"),
                Message::Close(_) => panic!("unexpected close"),
                _ => {}
            }
        }
    })
    .await
    .expect("frame timeout")
}

async fn next_close(client: &mut Client) -> (u16, String) {
    timeout(Duration::from_secs(2), async {
        loop {
            let message = client.next().await.expect("socket open").expect("message");
            if let Message::Close(frame) = message {
                let frame = frame.expect("close frame");
                return (u16::from(frame.code), frame.reason.to_string());
            }
        }
    })
    .await
    .expect("close timeout")
}

async fn pair(relay: &LocalRelay) -> (Client, Client, String, String) {
    let mut controller = connect(&relay.controller_url()).await;
    let RelayFrame::Hello { client_id: target } = next_frame(&mut controller).await else {
        panic!("expected hello");
    };
    let mut app = connect(&relay.pairing_url(&target)).await;
    let RelayFrame::ClientAttached { client_id: app_id } = next_frame(&mut controller).await else {
        panic!("expected client_attached");
    };
    let RelayFrame::ControllerAttached { client_id } = next_frame(&mut app).await else {
        panic!("expected controller_attached");
    };
    assert_eq!(client_id, target);
    (controller, app, target, app_id)
}

#[tokio::test]
async fn pairs_and_forwards_messages() {
    let relay = LocalRelay::start(config()).await.expect("relay");
    let (mut controller, mut app, _target, app_id) = pair(&relay).await;

    send(
        &mut controller,
        &RelayFrame::Message {
            client_id: Some(app_id.clone()),
            data: serde_json::json!({"t": "req", "reqId": "1", "m": "devices.get"}),
        },
    )
    .await;
    let RelayFrame::Message { client_id, data } = next_frame(&mut app).await else {
        panic!("expected app message");
    };
    assert_eq!(client_id, Some(app_id.clone()));
    assert_eq!(data["m"], "devices.get");

    send(
        &mut app,
        &RelayFrame::Message {
            client_id: None,
            data: serde_json::json!({"t": "resp", "reqId": "1", "result": {}}),
        },
    )
    .await;
    let RelayFrame::Message { client_id, data } = next_frame(&mut controller).await else {
        panic!("expected controller message");
    };
    assert_eq!(client_id, Some(app_id));
    assert_eq!(data["t"], "resp");

    relay.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn multiple_apps_and_disconnect_notifications() {
    let relay = LocalRelay::start(config()).await.expect("relay");
    let (mut controller, mut first_app, target, first_id) = pair(&relay).await;

    let mut second_app = connect(&relay.pairing_url(&target)).await;
    let RelayFrame::ClientAttached {
        client_id: second_id,
    } = next_frame(&mut controller).await
    else {
        panic!("expected second client_attached");
    };
    let RelayFrame::ControllerAttached { client_id } = next_frame(&mut second_app).await else {
        panic!("expected second controller_attached");
    };
    assert_eq!(client_id, target);
    assert_ne!(first_id, second_id);

    first_app.close(None).await.expect("close first app");
    let RelayFrame::ClientDisconnected { client_id } = next_frame(&mut controller).await else {
        panic!("expected client_disconnected");
    };
    assert_eq!(client_id, first_id);

    send(
        &mut controller,
        &RelayFrame::Message {
            client_id: Some(second_id.clone()),
            data: serde_json::json!({"ok": true}),
        },
    )
    .await;
    let RelayFrame::Message { client_id, .. } = next_frame(&mut second_app).await else {
        panic!("expected second app message");
    };
    assert_eq!(client_id, Some(second_id));

    relay.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn rejects_unknown_target_id() {
    let relay = LocalRelay::start(config()).await.expect("relay");
    let mut app = connect(&relay.pairing_url("missing")).await;
    let RelayFrame::Error { code, .. } = next_frame(&mut app).await else {
        panic!("expected error frame");
    };
    assert_eq!(code.as_deref(), Some("controller_not_found"));
    let (code, _) = next_close(&mut app).await;
    assert_eq!(code, 4001);
    relay.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn sends_heartbeat() {
    let relay = LocalRelay::start(RelayConfig {
        heartbeat_interval: Duration::from_millis(20),
        ..config()
    })
    .await
    .expect("relay");
    let mut controller = connect(&relay.controller_url()).await;
    let RelayFrame::Hello { .. } = next_frame(&mut controller).await else {
        panic!("expected hello");
    };
    let frame = next_frame(&mut controller).await;
    assert!(matches!(frame, RelayFrame::Heartbeat));
    relay.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn idle_timeout_closes_controller() {
    let relay = LocalRelay::start(RelayConfig {
        idle_timeout: Some(Duration::from_millis(100)),
        ..config()
    })
    .await
    .expect("relay");
    let mut controller = connect(&relay.controller_url()).await;
    let RelayFrame::Hello { .. } = next_frame(&mut controller).await else {
        panic!("expected hello");
    };
    let RelayFrame::IdleTimeout = next_frame(&mut controller).await else {
        panic!("expected idle_timeout");
    };
    let (code, _) = next_close(&mut controller).await;
    assert_eq!(code, 4002);
    relay.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn rejects_too_many_apps() {
    let relay = LocalRelay::start(RelayConfig {
        max_apps_per_controller: 1,
        ..config()
    })
    .await
    .expect("relay");
    let (_controller, _first_app, target, _) = pair(&relay).await;
    let mut second_app = connect(&relay.pairing_url(&target)).await;
    let RelayFrame::Error { code, .. } = next_frame(&mut second_app).await else {
        panic!("expected error");
    };
    assert_eq!(code.as_deref(), Some("too_many_apps"));
    let (code, _) = next_close(&mut second_app).await;
    assert_eq!(code, 1013);
    relay.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn malformed_frame_does_not_kill_connection() {
    let relay = LocalRelay::start(config()).await.expect("relay");
    let mut controller = connect(&relay.controller_url()).await;
    let RelayFrame::Hello { .. } = next_frame(&mut controller).await else {
        panic!("expected hello");
    };
    controller
        .send(Message::Text("{not json".into()))
        .await
        .expect("send malformed");
    send(&mut controller, &RelayFrame::Ping).await;
    let RelayFrame::Pong { .. } = next_frame(&mut controller).await else {
        panic!("expected pong");
    };
    relay.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn shutdown_closes_active_connections() {
    let relay = LocalRelay::start(config()).await.expect("relay");
    let mut controller = connect(&relay.controller_url()).await;
    let RelayFrame::Hello { .. } = next_frame(&mut controller).await else {
        panic!("expected hello");
    };
    relay.shutdown().await.expect("shutdown");
    let (code, _) = next_close(&mut controller).await;
    assert_eq!(code, 4000);
}
