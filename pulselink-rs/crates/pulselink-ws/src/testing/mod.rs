//! A local WebSocket relay for integration tests.

use std::sync::Arc;

use pulselink_protocol::socket::v4::{AppMessage, OperationEndReason, RelayFrame, RemoteDevice};
use tokio::net::TcpListener;
use tokio::sync::{Mutex, mpsc};

use crate::error::WsError;

mod command;
mod frames;
mod handler;
mod server;

use command::RelayCommand;
use server::server;

/// A scriptable local relay used by `pulselink-ws` tests.
#[derive(Clone)]
pub struct MockRelay {
    url: String,
    commands: mpsc::Sender<RelayCommand>,
    received: Arc<Mutex<mpsc::Receiver<RelayFrame>>>,
}

impl MockRelay {
    /// Starts a relay on an ephemeral localhost port.
    pub async fn start() -> Result<Self, WsError> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|error| WsError::Transport(error.to_string()))?;
        let address = listener
            .local_addr()
            .map_err(|error| WsError::Transport(error.to_string()))?;
        let (commands, command_rx) = mpsc::channel(64);
        let (received_tx, received_rx) = mpsc::channel(64);
        tokio::spawn(server(listener, command_rx, received_tx));
        Ok(Self {
            url: format!("ws://{address}"),
            commands,
            received: Arc::new(Mutex::new(received_rx)),
        })
    }

    /// Returns the relay's WebSocket URL.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Attaches an app client and sends its initial device snapshot.
    pub async fn attach_client(&self, client_id: &str, devices: Vec<RemoteDevice>) {
        let _ = self
            .commands
            .send(RelayCommand::AttachClient {
                client_id: client_id.to_owned(),
                devices,
            })
            .await;
    }

    /// Detaches an app client.
    pub async fn detach_client(&self, client_id: &str) {
        let _ = self
            .commands
            .send(RelayCommand::DetachClient {
                client_id: client_id.to_owned(),
            })
            .await;
    }

    /// Sends an app message to the controller.
    pub async fn send_app_message(&self, client_id: &str, message: AppMessage) {
        let _ = self
            .commands
            .send(RelayCommand::SendAppMessage {
                client_id: client_id.to_owned(),
                message,
            })
            .await;
    }

    /// Sends a server heartbeat.
    pub async fn heartbeat(&self) {
        let _ = self.commands.send(RelayCommand::Heartbeat).await;
    }

    /// Sends a server-level pong.
    pub async fn pong(&self, ts: i64) {
        let _ = self.commands.send(RelayCommand::Pong { ts }).await;
    }

    /// Sends an idle-timeout frame.
    pub async fn idle_timeout(&self) {
        let _ = self.commands.send(RelayCommand::IdleTimeout).await;
    }

    /// Sends a WebSocket close frame.
    pub async fn close(&self, code: u16, reason: &str) {
        let _ = self
            .commands
            .send(RelayCommand::Close {
                code,
                reason: reason.to_owned(),
            })
            .await;
    }

    /// Defers `device.op` responses until a clear or explicit completion.
    pub async fn set_defer_operate(&self, defer: bool) {
        let _ = self
            .commands
            .send(RelayCommand::SetDeferOperate { defer })
            .await;
    }

    /// Stops responding to RPC requests.
    pub async fn set_no_response(&self, no_response: bool) {
        let _ = self
            .commands
            .send(RelayCommand::SetNoResponse { no_response })
            .await;
    }

    /// Makes every RPC return the given error string.
    pub async fn set_rpc_error(&self, error: Option<String>) {
        let _ = self
            .commands
            .send(RelayCommand::SetRpcError { error })
            .await;
    }

    /// Completes a deferred operation manually.
    pub async fn complete_operation(&self, req_id: &str, reason: OperationEndReason) {
        let _ = self
            .commands
            .send(RelayCommand::CompleteOperation {
                req_id: req_id.to_owned(),
                reason,
            })
            .await;
    }

    /// Receives the next relay frame sent by the controller.
    pub async fn recv(&self) -> Option<RelayFrame> {
        self.received.lock().await.recv().await
    }
}
