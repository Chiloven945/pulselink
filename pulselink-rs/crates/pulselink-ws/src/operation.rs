//! Device operation submission and completion.

use std::sync::Arc;
use std::time::Duration;

use pulselink_core::Channel;
use pulselink_protocol::socket::v4::{ActionType, DeviceOperate, DeviceOperateResult};
use serde_json::Value;
use tokio::sync::oneshot;
use tokio::time::timeout;

use crate::error::WsError;
use crate::rpc::RpcManager;

/// The scope of a `device.op.clear` request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClearTarget {
    /// Clear every task on every device.
    All,
    /// Clear every channel of one device.
    Device {
        /// Device slot id.
        slot_id: String,
    },
    /// Clear one channel of one device.
    Channel {
        /// Device slot id.
        slot_id: String,
        /// Channel A or B.
        channel: Channel,
    },
}

impl ClearTarget {
    pub(crate) fn to_data(&self) -> Result<Option<Value>, WsError> {
        match self {
            Self::All => Ok(None),
            Self::Device { slot_id } => {
                if slot_id.is_empty() {
                    return Err(WsError::InvalidParams(
                        "clear slotId must not be empty".to_owned(),
                    ));
                }
                Ok(Some(serde_json::json!({ "s": slot_id })))
            }
            Self::Channel { slot_id, channel } => {
                if slot_id.is_empty() {
                    return Err(WsError::InvalidParams(
                        "clear slotId must not be empty".to_owned(),
                    ));
                }
                let channel = match channel {
                    Channel::A => 0,
                    Channel::B => 1,
                };
                Ok(Some(serde_json::json!({ "s": slot_id, "c": channel })))
            }
        }
    }
}

/// A submitted `device.op` task waiting for its completion response.
pub struct PendingOperation {
    /// Request id also used as the app-side task id.
    pub req_id: String,
    /// Operation type that was submitted.
    pub action: ActionType,
    pub(crate) receiver: oneshot::Receiver<Result<Value, WsError>>,
    pub(crate) timeout: Option<Duration>,
    pub(crate) rpc: Arc<RpcManager>,
    completed: bool,
}

impl PendingOperation {
    pub(crate) fn new(
        req_id: String,
        action: ActionType,
        receiver: oneshot::Receiver<Result<Value, WsError>>,
        timeout: Option<Duration>,
        rpc: Arc<RpcManager>,
    ) -> Self {
        Self {
            req_id,
            action,
            receiver,
            timeout,
            rpc,
            completed: false,
        }
    }

    /// Waits for the task to complete, be cleared, replaced, or cancelled.
    pub async fn completion(mut self) -> Result<DeviceOperateResult, WsError> {
        let req_id = self.req_id.clone();
        let result = if let Some(duration) = self.timeout {
            if let Ok(result) = timeout(duration, &mut self.receiver).await {
                result
            } else {
                let _ = self.rpc.remove(&self.req_id);
                return Err(WsError::RpcTimeout { req_id });
            }
        } else {
            (&mut self.receiver).await
        };

        let result = result.map_err(|_| WsError::Disconnected)?;
        self.completed = true;
        let value = result?;
        serde_json::from_value(value).map_err(WsError::from)
    }
}

impl Drop for PendingOperation {
    fn drop(&mut self) {
        if !self.completed {
            let _ = self.rpc.remove(&self.req_id);
        }
    }
}

pub(crate) fn validate_operate(operate: &DeviceOperate) -> Result<(), WsError> {
    operate
        .validate()
        .map_err(|error| WsError::InvalidParams(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_all_has_no_data() {
        assert_eq!(ClearTarget::All.to_data().expect("data"), None);
    }

    #[test]
    fn clear_device_and_channel_map_to_protocol() {
        let device = ClearTarget::Device {
            slot_id: "slot-a".to_owned(),
        }
        .to_data()
        .expect("device data")
        .expect("some data");
        assert_eq!(device["s"], "slot-a");
        assert!(device.get("c").is_none());

        let channel = ClearTarget::Channel {
            slot_id: "slot-a".to_owned(),
            channel: Channel::B,
        }
        .to_data()
        .expect("channel data")
        .expect("some data");
        assert_eq!(channel["s"], "slot-a");
        assert_eq!(channel["c"], 1);
    }
}
