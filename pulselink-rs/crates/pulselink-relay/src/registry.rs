//! Controller/app registry and routing table.

use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use pulselink_protocol::socket::v4::RelayFrame;
use tokio::sync::{broadcast, mpsc};

use crate::{RelayConfig, RelayEvent};

/// A command sent to a connection writer task.
#[derive(Clone, Debug)]
pub(crate) enum ServerCommand {
    Frame(RelayFrame),
    Close { code: u16, reason: String },
}

pub(crate) struct AppEntry {
    pub(crate) outbound: mpsc::Sender<ServerCommand>,
}

pub(crate) struct ControllerEntry {
    pub(crate) outbound: mpsc::Sender<ServerCommand>,
    pub(crate) apps: HashMap<String, AppEntry>,
    pub(crate) last_app_at: Instant,
}

pub(crate) struct RelayState {
    pub(crate) config: RelayConfig,
    controllers: RwLock<HashMap<String, ControllerEntry>>,
    next_client_id: AtomicU64,
    events: broadcast::Sender<RelayEvent>,
}

pub(crate) enum AttachError {
    ControllerNotFound,
    TooManyApps,
}

pub(crate) struct ControllerRemoval {
    pub(crate) apps: Vec<(String, mpsc::Sender<ServerCommand>)>,
}

impl RelayState {
    pub(crate) fn new(config: RelayConfig, events: broadcast::Sender<RelayEvent>) -> Self {
        Self {
            config,
            controllers: RwLock::new(HashMap::new()),
            next_client_id: AtomicU64::new(0),
            events,
        }
    }

    pub(crate) fn register_controller(
        &self,
        target_id: String,
        outbound: mpsc::Sender<ServerCommand>,
    ) {
        let now = Instant::now();
        self.controllers
            .write()
            .expect("relay registry lock poisoned")
            .insert(
                target_id.clone(),
                ControllerEntry {
                    outbound,
                    apps: HashMap::new(),
                    last_app_at: now,
                },
            );
        let _ = self
            .events
            .send(RelayEvent::ControllerConnected { target_id });
    }

    pub(crate) fn shutdown_all(&self) {
        let controllers = std::mem::take(
            &mut *self
                .controllers
                .write()
                .expect("relay registry lock poisoned"),
        );
        for controller in controllers.into_values() {
            let _ = controller.outbound.try_send(ServerCommand::Close {
                code: 4000,
                reason: "relay shutdown".to_owned(),
            });
            for app in controller.apps.into_values() {
                let _ = app.outbound.try_send(ServerCommand::Close {
                    code: 4000,
                    reason: "relay shutdown".to_owned(),
                });
            }
        }
    }

    pub(crate) fn remove_controller(&self, target_id: &str) -> Option<ControllerRemoval> {
        let entry = self
            .controllers
            .write()
            .expect("relay registry lock poisoned")
            .remove(target_id)?;
        let _ = self.events.send(RelayEvent::ControllerDisconnected {
            target_id: target_id.to_owned(),
        });
        Some(ControllerRemoval {
            apps: entry
                .apps
                .into_iter()
                .map(|(client_id, app)| (client_id, app.outbound))
                .collect(),
        })
    }

    pub(crate) fn attach_app(
        &self,
        target_id: &str,
        outbound: &mpsc::Sender<ServerCommand>,
    ) -> Result<String, AttachError> {
        let mut controllers = self
            .controllers
            .write()
            .expect("relay registry lock poisoned");
        let Some(controller) = controllers.get_mut(target_id) else {
            return Err(AttachError::ControllerNotFound);
        };
        if controller.apps.len() >= self.config.max_apps_per_controller {
            return Err(AttachError::TooManyApps);
        }

        let client_id = format!(
            "app-{}",
            self.next_client_id.fetch_add(1, Ordering::Relaxed) + 1
        );
        controller.apps.insert(
            client_id.clone(),
            AppEntry {
                outbound: outbound.clone(),
            },
        );
        controller.last_app_at = Instant::now();
        let controller_outbound = controller.outbound.clone();
        drop(controllers);

        send(
            &controller_outbound,
            RelayFrame::ClientAttached {
                client_id: client_id.clone(),
            },
        );
        send(
            outbound,
            RelayFrame::ControllerAttached {
                client_id: target_id.to_owned(),
            },
        );
        let _ = self.events.send(RelayEvent::AppAttached {
            target_id: target_id.to_owned(),
            client_id: client_id.clone(),
        });
        Ok(client_id)
    }

    pub(crate) fn detach_app(&self, target_id: &str, client_id: &str) {
        let mut controllers = self
            .controllers
            .write()
            .expect("relay registry lock poisoned");
        let Some(controller) = controllers.get_mut(target_id) else {
            return;
        };
        if controller.apps.remove(client_id).is_none() {
            return;
        }
        controller.last_app_at = Instant::now();
        let controller_outbound = controller.outbound.clone();
        drop(controllers);

        send(
            &controller_outbound,
            RelayFrame::ClientDisconnected {
                client_id: client_id.to_owned(),
            },
        );
        let _ = self.events.send(RelayEvent::AppDisconnected {
            target_id: target_id.to_owned(),
            client_id: client_id.to_owned(),
        });
    }

    pub(crate) fn route_from_controller(
        &self,
        target_id: &str,
        client_id: &str,
        data: serde_json::Value,
    ) -> bool {
        let outbound = self
            .controllers
            .read()
            .expect("relay registry lock poisoned")
            .get(target_id)
            .and_then(|controller| controller.apps.get(client_id))
            .map(|app| app.outbound.clone());
        let Some(outbound) = outbound else {
            return false;
        };
        send(
            &outbound,
            RelayFrame::Message {
                client_id: Some(client_id.to_owned()),
                data,
            },
        );
        true
    }

    pub(crate) fn route_from_app(
        &self,
        target_id: &str,
        client_id: &str,
        data: serde_json::Value,
    ) -> bool {
        let outbound = self
            .controllers
            .read()
            .expect("relay registry lock poisoned")
            .get(target_id)
            .map(|controller| controller.outbound.clone());
        let Some(outbound) = outbound else {
            return false;
        };
        send(
            &outbound,
            RelayFrame::Message {
                client_id: Some(client_id.to_owned()),
                data,
            },
        );
        true
    }

    pub(crate) fn controller_outbound(
        &self,
        target_id: &str,
    ) -> Option<mpsc::Sender<ServerCommand>> {
        self.controllers
            .read()
            .expect("relay registry lock poisoned")
            .get(target_id)
            .map(|controller| controller.outbound.clone())
    }

    pub(crate) fn controller_has_apps(&self, target_id: &str) -> bool {
        self.controllers
            .read()
            .expect("relay registry lock poisoned")
            .get(target_id)
            .is_some_and(|controller| !controller.apps.is_empty())
    }

    pub(crate) fn controller_idle_since(&self, target_id: &str) -> Option<Instant> {
        self.controllers
            .read()
            .expect("relay registry lock poisoned")
            .get(target_id)
            .map(|controller| controller.last_app_at)
    }
}

pub(crate) fn send(outbound: &mpsc::Sender<ServerCommand>, frame: RelayFrame) {
    let _ = outbound.try_send(ServerCommand::Frame(frame));
}
