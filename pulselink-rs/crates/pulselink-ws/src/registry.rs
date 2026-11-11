//! Registry of app clients attached to one controller socket.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Notify;

use crate::app::AppClient;

/// Maintains app clients in attach order.
#[derive(Default)]
pub(crate) struct Registry {
    apps: HashMap<String, AppClient>,
    order: Vec<String>,
    attach_notify: Arc<Notify>,
}

impl Registry {
    pub(crate) fn insert(&mut self, client: AppClient) -> Option<AppClient> {
        let client_id = client.client_id().to_owned();
        if !self.apps.contains_key(&client_id) {
            self.order.push(client_id.clone());
        }
        let previous = self.apps.insert(client_id, client);
        self.attach_notify.notify_waiters();
        previous
    }

    pub(crate) fn remove(&mut self, client_id: &str) -> Option<AppClient> {
        self.order.retain(|existing| existing != client_id);
        self.apps.remove(client_id)
    }

    pub(crate) fn get(&self, client_id: &str) -> Option<AppClient> {
        self.apps.get(client_id).cloned()
    }

    pub(crate) fn clients(&self) -> Vec<AppClient> {
        self.order
            .iter()
            .filter_map(|client_id| self.apps.get(client_id))
            .cloned()
            .collect()
    }

    pub(crate) fn first(&self) -> Option<AppClient> {
        self.order
            .first()
            .and_then(|client_id| self.apps.get(client_id))
            .cloned()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.apps.is_empty()
    }

    pub(crate) fn drain(&mut self) -> Vec<AppClient> {
        self.order.clear();
        self.apps.drain().map(|(_, client)| client).collect()
    }

    pub(crate) fn notify(&self) -> Arc<Notify> {
        Arc::clone(&self.attach_notify)
    }
}
