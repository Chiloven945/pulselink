//! DG-LAB V4 inner RPC envelopes.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::socket::v4::event::AppEvent;

/// An application-level message carried inside a relay `message` frame.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "t")]
pub enum AppMessage {
    /// Controller-to-app RPC request.
    #[serde(rename = "req")]
    Req(RpcRequest),
    /// App-to-controller RPC response.
    #[serde(rename = "resp")]
    Resp(RpcResponse),
    /// App-to-controller event.
    #[serde(rename = "ev")]
    Ev(AppEvent),
}

/// An RPC request.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RpcRequest {
    /// Request id, unique among pending requests for the same app client.
    #[serde(rename = "reqId")]
    pub req_id: String,
    /// Method name.
    pub m: String,
    /// Optional method parameters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// An RPC response.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RpcResponse {
    /// Request id being answered.
    #[serde(rename = "reqId")]
    pub req_id: String,
    /// Successful result, mutually exclusive with `error`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error code or reason, mutually exclusive with `result`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_request() {
        let message: AppMessage =
            serde_json::from_str(r#"{"t":"req","reqId":"req-1","m":"devices.get"}"#)
                .expect("valid request");
        assert_eq!(
            message,
            AppMessage::Req(RpcRequest {
                req_id: "req-1".to_owned(),
                m: "devices.get".to_owned(),
                data: None,
            })
        );
    }

    #[test]
    fn parses_response() {
        let success: AppMessage =
            serde_json::from_str(r#"{"t":"resp","reqId":"req-1","result":{"devices":[]}}"#)
                .expect("valid success response");
        assert!(matches!(
            success,
            AppMessage::Resp(RpcResponse { error: None, .. })
        ));

        let failure: AppMessage =
            serde_json::from_str(r#"{"t":"resp","reqId":"req-2","error":"slot_not_found"}"#)
                .expect("valid error response");
        assert!(matches!(
            failure,
            AppMessage::Resp(RpcResponse {
                error: Some(_),
                result: None,
                ..
            })
        ));
    }

    #[test]
    fn parses_event() {
        let message: AppMessage =
            serde_json::from_str(r#"{"t":"ev","ev":"devices.snapshot","devices":[]}"#)
                .expect("valid event");
        assert_eq!(
            message,
            AppMessage::Ev(AppEvent::DevicesSnapshot {
                devices: Vec::new()
            })
        );
    }

    #[test]
    fn serializes_request() {
        let message = AppMessage::Req(RpcRequest {
            req_id: "req-1".to_owned(),
            m: "device.op".to_owned(),
            data: Some(serde_json::json!({"s": "slot-a"})),
        });
        let json = serde_json::to_value(&message).expect("serialize");
        assert_eq!(json["t"], "req");
        assert_eq!(json["reqId"], "req-1");
        assert_eq!(json["data"]["s"], "slot-a");
    }
}
