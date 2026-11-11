//! Shared socket state, app registry, and frame routing.

use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::time::Duration;

use pulselink_protocol::socket::v4::{AppMessage, RelayFrame, RpcResponse};
use serde_json::Value;
use tokio::sync::{broadcast, mpsc, oneshot, watch};
use tracing::debug;

use crate::app::AppClient;
use crate::error::WsError;
use crate::registry::Registry;
use crate::transport::Outbound;

use super::state::{SocketEvent, SocketState};

pub(super) struct SocketInner {
    pub(super) target_id: OnceLock<String>,
    pub(super) state_tx: watch::Sender<SocketState>,
    pub(super) writer: mpsc::Sender<Outbound>,
    pub(super) events: broadcast::Sender<SocketEvent>,
    pub(super) registry: RwLock<Registry>,
    pub(super) next_req_id: Arc<AtomicU64>,
    pub(super) shutdown_tx: watch::Sender<bool>,
    pub(super) response_timeout: Duration,
    pub(super) pong_waiter: Mutex<Option<oneshot::Sender<()>>>,
}

impl SocketInner {
    pub(super) fn new(
        writer: mpsc::Sender<Outbound>,
        shutdown_tx: watch::Sender<bool>,
        response_timeout: Duration,
    ) -> Self {
        let (state_tx, _) = watch::channel(SocketState::Connecting);
        let (events, _) = broadcast::channel(256);
        Self {
            target_id: OnceLock::new(),
            state_tx,
            writer,
            events,
            registry: RwLock::new(Registry::default()),
            next_req_id: Arc::new(AtomicU64::new(0)),
            shutdown_tx,
            response_timeout,
            pong_waiter: Mutex::new(None),
        }
    }

    pub(crate) fn state(&self) -> SocketState {
        *self.state_tx.subscribe().borrow()
    }

    pub(super) fn set_state(&self, state: SocketState) {
        self.state_tx.send_replace(state);
        self.emit(SocketEvent::StateChanged { state });
    }

    pub(super) fn emit(&self, event: SocketEvent) {
        let _ = self.events.send(event);
    }

    pub(super) fn register_client(&self, client_id: String) {
        let first = self
            .registry
            .read()
            .expect("registry lock poisoned")
            .is_empty();
        let client = AppClient::new(
            client_id.clone(),
            self.writer.clone(),
            Arc::clone(&self.next_req_id),
            self.response_timeout,
        );
        let previous = self
            .registry
            .write()
            .expect("registry lock poisoned")
            .insert(client);
        if let Some(previous) = previous {
            previous.disconnect();
        }
        if first {
            self.set_state(SocketState::Ready);
        }
        self.emit(SocketEvent::ClientAttached { client_id });
    }

    pub(super) fn remove_client(&self, client_id: &str) {
        let removed = self
            .registry
            .write()
            .expect("registry lock poisoned")
            .remove(client_id);
        if let Some(client) = removed {
            client.disconnect();
            self.emit(SocketEvent::ClientDisconnected {
                client_id: client_id.to_owned(),
            });
            let empty = self
                .registry
                .read()
                .expect("registry lock poisoned")
                .is_empty();
            if empty && self.state() != SocketState::Closed {
                self.set_state(SocketState::WaitingForPeer);
            }
        }
    }

    pub(super) fn close_clients(&self, error: &WsError) {
        let clients = self
            .registry
            .write()
            .expect("registry lock poisoned")
            .drain();
        for client in clients {
            client.fail_all(error);
            client.disconnect();
        }
    }

    pub(super) fn close_with(&self, code: Option<u16>, reason: String) {
        if self.state() == SocketState::Closed {
            return;
        }
        self.set_state(SocketState::Closed);
        self.close_clients(&WsError::Disconnected);
        self.emit(SocketEvent::Closed { code, reason });
        let _ = self.shutdown_tx.send(true);
    }

    pub(super) fn fail_transport(&self, message: String) {
        if self.state() == SocketState::Closed {
            return;
        }
        self.emit(SocketEvent::Error {
            message: message.clone(),
        });
        self.close_with(None, message);
    }

    pub(super) fn notify_pong(&self) {
        if let Some(waiter) = self
            .pong_waiter
            .lock()
            .expect("pong waiter mutex poisoned")
            .take()
        {
            let _ = waiter.send(());
        }
    }

    pub(super) fn handle_text(&self, text: &str, hello: &mut Option<oneshot::Sender<()>>) {
        let frame: RelayFrame = match serde_json::from_str(text) {
            Ok(frame) => frame,
            Err(error) => {
                self.emit(SocketEvent::Error {
                    message: format!("invalid relay frame: {error}"),
                });
                return;
            }
        };

        match frame {
            RelayFrame::Hello { client_id } => {
                if self.target_id.set(client_id).is_err() {
                    debug!("ignoring duplicate hello");
                    return;
                }
                self.set_state(SocketState::WaitingForPeer);
                if let Some(sender) = hello.take() {
                    let _ = sender.send(());
                }
            }
            RelayFrame::ClientAttached { client_id } => self.register_client(client_id),
            RelayFrame::ClientDisconnected { client_id } => self.remove_client(&client_id),
            RelayFrame::Message { client_id, data } => {
                self.route_message(client_id.as_deref(), data);
            }
            RelayFrame::Heartbeat => self.emit(SocketEvent::Heartbeat),
            RelayFrame::Pong { ts } => {
                self.notify_pong();
                self.emit(SocketEvent::Pong { ts });
            }
            RelayFrame::IdleTimeout => {
                self.emit(SocketEvent::IdleTimeout);
                self.close_with(None, "idle timeout".to_owned());
            }
            RelayFrame::Error { code, message } => {
                self.emit(SocketEvent::Error {
                    message: format!(
                        "relay error {}{}",
                        code.unwrap_or_else(|| "unknown".to_owned()),
                        message.map_or_else(String::new, |message| format!(": {message}"))
                    ),
                });
            }
            RelayFrame::Ping
            | RelayFrame::ControllerAttached { .. }
            | RelayFrame::ControllerDisconnected { .. }
            | RelayFrame::Unknown => {}
        }
    }

    pub(super) fn route_message(&self, client_id: Option<&str>, data: Value) {
        let message: AppMessage = match serde_json::from_value(data) {
            Ok(message) => message,
            Err(error) => {
                self.emit(SocketEvent::Error {
                    message: format!("invalid app message: {error}"),
                });
                return;
            }
        };

        match message {
            AppMessage::Resp(response) => {
                self.handle_response(client_id, response);
            }
            AppMessage::Ev(event) => {
                let app = client_id
                    .and_then(|client_id| self.app(client_id))
                    .or_else(|| self.single_app());
                if let Some(app) = app {
                    app.apply_event(event);
                } else {
                    self.emit(SocketEvent::Error {
                        message: "event has no matching app client".to_owned(),
                    });
                }
            }
            AppMessage::Req(request) => {
                self.emit(SocketEvent::Error {
                    message: format!("unexpected app request: {}", request.m),
                });
            }
        }
    }

    pub(super) fn handle_response(&self, client_id: Option<&str>, response: RpcResponse) {
        let app = client_id
            .and_then(|client_id| self.app(client_id))
            .filter(|app| app.pending_method(&response.req_id).is_some())
            .or_else(|| self.app_by_req_id(&response.req_id));
        let Some(app) = app else {
            self.emit(SocketEvent::Error {
                message: format!("response for unknown client: {}", response.req_id),
            });
            return;
        };

        let method = app
            .pending_method(&response.req_id)
            .unwrap_or_else(|| "unknown".to_owned());
        let result = if let Some(error) = response.error {
            Err(WsError::from_rpc(error, method))
        } else if let Some(result) = response.result {
            Ok(result)
        } else {
            Err(WsError::InvalidParams(
                "RPC response has neither result nor error".to_owned(),
            ))
        };

        if !app.complete_rpc(&response.req_id, result) {
            self.emit(SocketEvent::Error {
                message: format!("response for unknown request: {}", response.req_id),
            });
        }
    }

    pub(super) fn app(&self, client_id: &str) -> Option<AppClient> {
        self.registry
            .read()
            .expect("registry lock poisoned")
            .get(client_id)
    }

    pub(super) fn single_app(&self) -> Option<AppClient> {
        let clients = self
            .registry
            .read()
            .expect("registry lock poisoned")
            .clients();
        (clients.len() == 1).then(|| clients[0].clone())
    }

    pub(super) fn app_by_req_id(&self, req_id: &str) -> Option<AppClient> {
        let clients = self
            .registry
            .read()
            .expect("registry lock poisoned")
            .clients();
        let mut matches = clients
            .into_iter()
            .filter(|client| client.pending_method(req_id).is_some());
        let first = matches.next()?;
        if matches.next().is_some() {
            None
        } else {
            Some(first)
        }
    }
}
