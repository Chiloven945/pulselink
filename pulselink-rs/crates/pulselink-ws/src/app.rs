//! Per-app client handles, cache, and events.

use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use pulselink_protocol::socket::v4::{
    AppEvent, AppMessage, DeviceOperate, RelayFrame, RemoteDevice, RpcRequest, SlotPatchEntry,
};
use serde_json::Value;
use tokio::sync::{broadcast, mpsc};
use tokio::time::timeout;

use crate::device::{DeviceCache, RemoteDeviceHandle};
use crate::error::WsError;
use crate::operation::{ClearTarget, PendingOperation, validate_operate};
use crate::rpc::{PendingGuard, RpcManager};
use crate::transport::Outbound;

/// An event emitted by one app client.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum WsEvent {
    /// Full device snapshot received from the app.
    DevicesSnapshot {
        /// Devices in the app's list.
        devices: Vec<RemoteDevice>,
    },
    /// Incremental device list update.
    DevicesPatch {
        /// Added or replaced devices.
        added: Vec<RemoteDevice>,
        /// Removed slot ids.
        removed: Vec<String>,
    },
    /// Incremental slot state update.
    SlotsPatch {
        /// Changed slots.
        slots: Vec<SlotPatchEntry>,
    },
    /// A custom app action.
    CustomAction {
        /// Action number.
        action: u8,
    },
    /// The app client disconnected.
    Disconnected,
    /// A non-fatal protocol or routing error.
    Error {
        /// Error description.
        message: String,
    },
}

pub(crate) struct AppInner {
    pub(crate) client_id: String,
    pub(crate) writer: mpsc::Sender<Outbound>,
    pub(crate) rpc: Arc<RpcManager>,
    pub(crate) next_req_id: Arc<AtomicU64>,
    pub(crate) cache: RwLock<DeviceCache>,
    pub(crate) events: broadcast::Sender<WsEvent>,
    pub(crate) response_timeout: Duration,
}

/// A cloneable handle to one app client attached to a controller.
#[derive(Clone)]
pub struct AppClient {
    inner: Arc<AppInner>,
}

impl AppClient {
    pub(crate) fn new(
        client_id: String,
        writer: mpsc::Sender<Outbound>,
        next_req_id: Arc<AtomicU64>,
        response_timeout: Duration,
    ) -> Self {
        let (events, _) = broadcast::channel(256);
        Self {
            inner: Arc::new(AppInner {
                client_id,
                writer,
                rpc: Arc::new(RpcManager::default()),
                next_req_id,
                cache: RwLock::new(DeviceCache::default()),
                events,
                response_timeout,
            }),
        }
    }

    /// Returns the app's client id.
    #[must_use]
    pub fn client_id(&self) -> &str {
        &self.inner.client_id
    }

    /// Returns the last cached device snapshot.
    pub fn devices(&self) -> Vec<RemoteDevice> {
        self.inner
            .cache
            .read()
            .expect("device cache lock poisoned")
            .snapshot()
    }

    /// Requests a fresh device list from the app.
    pub async fn request_devices(&self) -> Result<Vec<RemoteDevice>, WsError> {
        let value = self.request("devices.get", None).await?;
        let devices: Vec<RemoteDevice> = value
            .get("devices")
            .cloned()
            .ok_or_else(|| WsError::InvalidParams("devices.get result has no devices".to_owned()))
            .and_then(|value| serde_json::from_value(value).map_err(WsError::from))?;
        let _ = self
            .inner
            .cache
            .write()
            .expect("device cache lock poisoned")
            .replace(devices.clone());
        Ok(devices)
    }

    /// Returns a snapshot of one cached device.
    #[must_use]
    pub fn device_snapshot(&self, slot_id: &str) -> Option<crate::device::RemoteDeviceSnapshot> {
        self.inner
            .cache
            .read()
            .expect("device cache lock poisoned")
            .snapshot_device(slot_id)
    }

    /// Returns a live device handle if the slot is currently cached.
    #[must_use]
    pub fn device(&self, slot_id: &str) -> Option<RemoteDeviceHandle> {
        let shared = self
            .inner
            .cache
            .read()
            .expect("device cache lock poisoned")
            .shared(slot_id)?;
        Some(RemoteDeviceHandle::new(
            self.clone(),
            shared,
            slot_id.to_owned(),
        ))
    }

    /// Subscribes to this app's events.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<WsEvent> {
        self.inner.events.subscribe()
    }

    /// Calls the app-level `ping` RPC and returns its local timestamp.
    pub async fn ping(&self) -> Result<i64, WsError> {
        let value = self.request("ping", None).await?;
        value
            .as_i64()
            .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
            .ok_or_else(|| WsError::InvalidParams("ping result is not an integer".to_owned()))
    }

    /// Clears device operations at the requested scope.
    pub async fn clear_operate(&self, target: ClearTarget) -> Result<(), WsError> {
        let data = target.to_data()?;
        let _ = self.request("device.op.clear", data).await?;
        Ok(())
    }

    /// Submits a raw `device.op` task and returns a handle for its completion.
    pub async fn submit_operate(
        &self,
        operate: DeviceOperate,
        operation_timeout: Option<Duration>,
    ) -> Result<PendingOperation, WsError> {
        validate_operate(&operate)?;
        let value = serde_json::to_value(&operate).map_err(WsError::from)?;
        let action = operate.action;
        let (req_id, receiver) = self
            .inner
            .rpc
            .submit(&self.inner.next_req_id, "device.op")?;
        if let Err(error) = self.send_request(&req_id, "device.op", Some(value)).await {
            let _ = self.inner.rpc.remove(&req_id);
            return Err(error);
        }

        Ok(PendingOperation::new(
            req_id,
            action,
            receiver,
            operation_timeout,
            Arc::clone(&self.inner.rpc),
        ))
    }

    pub(crate) async fn request(
        &self,
        method: &str,
        data: Option<Value>,
    ) -> Result<Value, WsError> {
        let (req_id, receiver) = self.inner.rpc.submit(&self.inner.next_req_id, method)?;
        let mut guard = PendingGuard::new(Arc::clone(&self.inner.rpc), req_id.clone());
        self.send_request(&req_id, method, data).await?;

        let result = match timeout(self.inner.response_timeout, receiver).await {
            Ok(result) => result.map_err(|_| WsError::Disconnected)?,
            Err(_) => {
                return Err(WsError::RpcTimeout {
                    req_id: req_id.clone(),
                });
            }
        };
        guard.disarm();
        result
    }

    async fn send_request(
        &self,
        req_id: &str,
        method: &str,
        data: Option<Value>,
    ) -> Result<(), WsError> {
        let message = AppMessage::Req(RpcRequest {
            req_id: req_id.to_owned(),
            m: method.to_owned(),
            data,
        });
        let frame = RelayFrame::Message {
            client_id: Some(self.inner.client_id.clone()),
            data: serde_json::to_value(message).map_err(WsError::from)?,
        };
        let text = serde_json::to_string(&frame).map_err(WsError::from)?;
        timeout(
            self.inner.response_timeout,
            self.inner.writer.send(Outbound::Text(text)),
        )
        .await
        .map_err(|_| WsError::Transport("writer queue timed out".to_owned()))?
        .map_err(|_| WsError::Disconnected)
    }

    pub(crate) fn apply_event(&self, event: AppEvent) {
        if let Err(error) = event.validate() {
            self.emit(WsEvent::Error {
                message: error.to_string(),
            });
            return;
        }

        match event {
            AppEvent::DevicesSnapshot { devices } => {
                let devices = self
                    .inner
                    .cache
                    .write()
                    .expect("device cache lock poisoned")
                    .replace(devices);
                self.emit(WsEvent::DevicesSnapshot { devices });
            }
            AppEvent::DevicesPatch { added, removed } => {
                let patch = pulselink_protocol::socket::v4::DevicePatch {
                    added: added.clone(),
                    removed: removed.clone(),
                };
                self.inner
                    .cache
                    .write()
                    .expect("device cache lock poisoned")
                    .apply_patch(&patch);
                self.emit(WsEvent::DevicesPatch { added, removed });
            }
            AppEvent::SlotsPatch { slots } => {
                self.inner
                    .cache
                    .write()
                    .expect("device cache lock poisoned")
                    .apply_slots_patch(&slots);
                self.emit(WsEvent::SlotsPatch { slots });
            }
            AppEvent::CustomAction { action } => {
                self.emit(WsEvent::CustomAction { action });
            }
            AppEvent::Unknown => {}
        }
    }

    pub(crate) fn complete_rpc(&self, req_id: &str, result: Result<Value, WsError>) -> bool {
        if self.inner.rpc.contains(req_id) {
            self.inner.rpc.complete(req_id, result);
            true
        } else {
            false
        }
    }

    pub(crate) fn pending_method(&self, req_id: &str) -> Option<String> {
        self.inner.rpc.method(req_id)
    }

    pub(crate) fn disconnect(&self) {
        self.inner.rpc.fail_all(&WsError::Disconnected);
        self.emit(WsEvent::Disconnected);
    }

    pub(crate) fn fail_all(&self, error: &WsError) {
        self.inner.rpc.fail_all(error);
    }

    pub(crate) fn emit(&self, event: WsEvent) {
        let _ = self.inner.events.send(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulselink_protocol::socket::v4::{AppEvent, RemoteDevice};
    use serde_json::{Map, json};

    #[tokio::test]
    async fn applies_snapshot_and_patch_to_cache() {
        let (writer, _receiver) = mpsc::channel(4);
        let client = AppClient::new(
            "app".to_owned(),
            writer,
            Arc::new(AtomicU64::new(0)),
            Duration::from_secs(1),
        );

        client.apply_event(AppEvent::DevicesSnapshot {
            devices: vec![RemoteDevice {
                id: None,
                slot_id: "a".to_owned(),
                name: "A".to_owned(),
                device_type: "COYOTE_030".to_owned(),
                props: serde_json::from_value(json!({"nested": {"a": 1}})).expect("object"),
                slot_state: Map::default(),
            }],
        });
        assert_eq!(client.devices().len(), 1);

        client.apply_event(AppEvent::SlotsPatch {
            slots: vec![pulselink_protocol::socket::v4::SlotPatchEntry {
                slot_id: "a".to_owned(),
                props: serde_json::from_value(json!({"nested": {"b": 2}})).expect("object"),
                slot_state: Map::default(),
            }],
        });
        let devices = client.devices();
        assert_eq!(devices[0].props["nested"]["a"], 1);
        assert_eq!(devices[0].props["nested"]["b"], 2);
    }
}
