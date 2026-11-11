//! RPC request id generation and pending response tracking.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::Value;
use tokio::sync::oneshot;

use crate::error::WsError;

const MAX_PENDING_RPCS: usize = 256;

pub(crate) struct PendingRpc {
    pub(crate) method: String,
    pub(crate) reply: oneshot::Sender<Result<Value, WsError>>,
}

/// Tracks pending RPC requests for one app client.
#[derive(Default)]
pub(crate) struct RpcManager {
    pending: Mutex<HashMap<String, PendingRpc>>,
}

impl RpcManager {
    /// Inserts a pending request and returns its unique id and receiver.
    pub(crate) fn submit(
        &self,
        next_id: &AtomicU64,
        method: &str,
    ) -> Result<(String, oneshot::Receiver<Result<Value, WsError>>), WsError> {
        let mut pending = self.pending.lock().expect("RPC pending mutex poisoned");
        if pending.len() >= MAX_PENDING_RPCS {
            return Err(WsError::InvalidParams(
                "too many pending RPC requests".to_owned(),
            ));
        }

        let req_id = format!("req-{}", next_id.fetch_add(1, Ordering::Relaxed) + 1);
        let (reply, receiver) = oneshot::channel();
        pending.insert(
            req_id.clone(),
            PendingRpc {
                method: method.to_owned(),
                reply,
            },
        );
        Ok((req_id, receiver))
    }

    /// Removes a pending request without completing it.
    pub(crate) fn remove(&self, req_id: &str) -> Option<PendingRpc> {
        self.pending
            .lock()
            .expect("RPC pending mutex poisoned")
            .remove(req_id)
    }

    /// Completes a pending request if it is still present.
    pub(crate) fn complete(&self, req_id: &str, result: Result<Value, WsError>) {
        if let Some(pending) = self.remove(req_id) {
            let _ = pending.reply.send(result);
        }
    }

    /// Completes every pending request with the given error.
    pub(crate) fn fail_all(&self, error: &WsError) {
        let pending = {
            let mut guard = self.pending.lock().expect("RPC pending mutex poisoned");
            std::mem::take(&mut *guard)
        };
        for (_, pending) in pending {
            let _ = pending.reply.send(Err(error.clone()));
        }
    }

    /// Returns whether a request id is currently pending.
    pub(crate) fn contains(&self, req_id: &str) -> bool {
        self.pending
            .lock()
            .expect("RPC pending mutex poisoned")
            .contains_key(req_id)
    }

    /// Returns the method associated with a request id.
    pub(crate) fn method(&self, req_id: &str) -> Option<String> {
        self.pending
            .lock()
            .expect("RPC pending mutex poisoned")
            .get(req_id)
            .map(|pending| pending.method.clone())
    }
}

/// A guard that removes a pending request if it is dropped without completion.
pub(crate) struct PendingGuard {
    manager: Arc<RpcManager>,
    pub(crate) req_id: String,
    armed: bool,
}

impl PendingGuard {
    pub(crate) fn new(manager: Arc<RpcManager>, req_id: String) -> Self {
        Self {
            manager,
            req_id,
            armed: true,
        }
    }

    pub(crate) fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for PendingGuard {
    fn drop(&mut self) {
        if self.armed {
            let _ = self.manager.remove(&self.req_id);
        }
    }
}
