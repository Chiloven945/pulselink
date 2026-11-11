//! DG-LAB V4 outer relay frames.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A frame exchanged with the V4 WebSocket relay.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RelayFrame {
    /// The server assigned a client id to this connection.
    Hello {
        /// Connection client id.
        #[serde(rename = "clientId")]
        client_id: String,
    },
    /// An app client attached to this controller.
    ClientAttached {
        /// App client id.
        #[serde(rename = "clientId")]
        client_id: String,
    },
    /// An app client disconnected.
    ClientDisconnected {
        /// App client id.
        #[serde(rename = "clientId")]
        client_id: String,
    },
    /// This app attached to a controller.
    ControllerAttached {
        /// Controller client id.
        #[serde(rename = "clientId")]
        client_id: String,
    },
    /// The controller disconnected.
    ControllerDisconnected {
        /// Controller client id.
        #[serde(rename = "clientId")]
        client_id: String,
    },
    /// Server heartbeat.
    Heartbeat,
    /// Server-level ping sent by a client.
    Ping,
    /// Server-level pong.
    Pong {
        /// Optional server timestamp.
        #[serde(default)]
        ts: Option<i64>,
    },
    /// An application message.
    Message {
        /// Target or source app client id, depending on direction.
        #[serde(default, rename = "clientId")]
        client_id: Option<String>,
        /// Application payload.
        data: Value,
    },
    /// The controller's idle timeout expired.
    IdleTimeout,
    /// Server error frame.
    Error {
        /// Optional error code.
        #[serde(default)]
        code: Option<String>,
        /// Optional human-readable error message.
        #[serde(default)]
        message: Option<String>,
    },
    /// A frame type unknown to this version of the protocol crate.
    #[serde(other)]
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hello() {
        let frame: RelayFrame =
            serde_json::from_str(r#"{"type":"hello","clientId":"controller-1"}"#)
                .expect("valid hello");
        assert_eq!(
            frame,
            RelayFrame::Hello {
                client_id: "controller-1".to_owned()
            }
        );
    }

    #[test]
    fn parses_client_lifecycle() {
        let attached: RelayFrame =
            serde_json::from_str(r#"{"type":"client_attached","clientId":"app-1"}"#)
                .expect("valid client_attached");
        assert_eq!(
            attached,
            RelayFrame::ClientAttached {
                client_id: "app-1".to_owned()
            }
        );

        let disconnected: RelayFrame =
            serde_json::from_str(r#"{"type":"client_disconnected","clientId":"app-1"}"#)
                .expect("valid client_disconnected");
        assert_eq!(
            disconnected,
            RelayFrame::ClientDisconnected {
                client_id: "app-1".to_owned()
            }
        );
    }

    #[test]
    fn parses_message_with_and_without_client_id() {
        let outbound: RelayFrame =
            serde_json::from_str(r#"{"type":"message","clientId":"app-1","data":{"t":"req"}}"#)
                .expect("valid outbound message");
        assert!(matches!(
            outbound,
            RelayFrame::Message {
                client_id: Some(_),
                ..
            }
        ));

        let inbound: RelayFrame = serde_json::from_str(r#"{"type":"message","data":{"t":"ev"}}"#)
            .expect("valid inbound message");
        assert!(matches!(
            inbound,
            RelayFrame::Message {
                client_id: None,
                ..
            }
        ));
    }

    #[test]
    fn parses_heartbeat_pong_idle_and_error() {
        assert_eq!(
            serde_json::from_str::<RelayFrame>(r#"{"type":"heartbeat"}"#).expect("heartbeat"),
            RelayFrame::Heartbeat
        );
        assert_eq!(
            serde_json::from_str::<RelayFrame>(r#"{"type":"pong","ts":123}"#).expect("pong"),
            RelayFrame::Pong { ts: Some(123) }
        );
        assert_eq!(
            serde_json::from_str::<RelayFrame>(r#"{"type":"idle_timeout"}"#).expect("idle_timeout"),
            RelayFrame::IdleTimeout
        );
        assert_eq!(
            serde_json::from_str::<RelayFrame>(
                r#"{"type":"error","code":"controller_not_found","message":"missing"}"#
            )
            .expect("error"),
            RelayFrame::Error {
                code: Some("controller_not_found".to_owned()),
                message: Some("missing".to_owned()),
            }
        );
    }

    #[test]
    fn unknown_type_is_forward_compatible() {
        assert_eq!(
            serde_json::from_str::<RelayFrame>(r#"{"type":"future_frame","x":1}"#)
                .expect("unknown frame"),
            RelayFrame::Unknown
        );
    }

    #[test]
    fn serializes_message() {
        let frame = RelayFrame::Message {
            client_id: Some("app-1".to_owned()),
            data: serde_json::json!({"t": "req", "reqId": "req-1", "m": "devices.get"}),
        };
        let json = serde_json::to_value(&frame).expect("serialize");
        assert_eq!(json["type"], "message");
        assert_eq!(json["clientId"], "app-1");
        assert_eq!(json["data"]["m"], "devices.get");
    }
}
