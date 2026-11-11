//! Controller socket, reader/writer tasks, and app routing.

use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::time::Duration;

use pulselink_protocol::socket::v4::{AppMessage, RelayFrame, RpcResponse};
use serde_json::Value;
use tokio::sync::{broadcast, mpsc, oneshot, watch};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tracing::debug;

use crate::app::{AppClient, WsEvent};
use crate::error::WsError;
use crate::options::SocketOptions;
use crate::registry::Registry;
use crate::transport::{Inbound, Outbound, TungsteniteTransport, WsSink, WsSource, WsTransport};

/// Connection state of a [`Socket`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SocketState {
    /// The transport handshake is in progress.
    Connecting,
    /// `hello` was received; the socket is waiting for an app to attach.
    WaitingForPeer,
    /// At least one app client is attached.
    Ready,
    /// A local shutdown is in progress.
    Closing,
    /// The socket is closed.
    Closed,
}

/// A lifecycle or control event emitted by a [`Socket`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum SocketEvent {
    /// The connection state changed.
    StateChanged {
        /// New state.
        state: SocketState,
    },
    /// An app attached.
    ClientAttached {
        /// App client id.
        client_id: String,
    },
    /// An app detached.
    ClientDisconnected {
        /// App client id.
        client_id: String,
    },
    /// A server heartbeat was received.
    Heartbeat,
    /// A server-level pong was received.
    Pong {
        /// Optional server timestamp.
        ts: Option<i64>,
    },
    /// The relay reported an idle timeout.
    IdleTimeout,
    /// The WebSocket closed.
    Closed {
        /// Optional WebSocket close code.
        code: Option<u16>,
        /// Close reason.
        reason: String,
    },
    /// A non-fatal protocol or transport error.
    Error {
        /// Error description.
        message: String,
    },
}

pub(crate) struct SocketInner {
    target_id: OnceLock<String>,
    state_tx: watch::Sender<SocketState>,
    writer: mpsc::Sender<Outbound>,
    events: broadcast::Sender<SocketEvent>,
    registry: RwLock<Registry>,
    next_req_id: Arc<AtomicU64>,
    shutdown_tx: watch::Sender<bool>,
    response_timeout: Duration,
    pong_waiter: Mutex<Option<oneshot::Sender<()>>>,
}

impl SocketInner {
    fn new(
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

    fn set_state(&self, state: SocketState) {
        self.state_tx.send_replace(state);
        self.emit(SocketEvent::StateChanged { state });
    }

    fn emit(&self, event: SocketEvent) {
        let _ = self.events.send(event);
    }

    fn register_client(&self, client_id: String) {
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

    fn remove_client(&self, client_id: &str) {
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

    fn close_clients(&self, error: &WsError) {
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

    fn close_with(&self, code: Option<u16>, reason: String) {
        if self.state() == SocketState::Closed {
            return;
        }
        self.set_state(SocketState::Closed);
        self.close_clients(&WsError::Disconnected);
        self.emit(SocketEvent::Closed { code, reason });
        let _ = self.shutdown_tx.send(true);
    }

    fn fail_transport(&self, message: String) {
        if self.state() == SocketState::Closed {
            return;
        }
        self.emit(SocketEvent::Error {
            message: message.clone(),
        });
        self.close_with(None, message);
    }

    fn notify_pong(&self) {
        if let Some(waiter) = self
            .pong_waiter
            .lock()
            .expect("pong waiter mutex poisoned")
            .take()
        {
            let _ = waiter.send(());
        }
    }

    fn handle_text(&self, text: &str, hello: &mut Option<oneshot::Sender<()>>) {
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

    fn route_message(&self, client_id: Option<&str>, data: Value) {
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

    fn handle_response(&self, client_id: Option<&str>, response: RpcResponse) {
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

    fn app(&self, client_id: &str) -> Option<AppClient> {
        self.registry
            .read()
            .expect("registry lock poisoned")
            .get(client_id)
    }

    fn single_app(&self) -> Option<AppClient> {
        let clients = self
            .registry
            .read()
            .expect("registry lock poisoned")
            .clients();
        (clients.len() == 1).then(|| clients[0].clone())
    }

    fn app_by_req_id(&self, req_id: &str) -> Option<AppClient> {
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

/// A cloneable DG-LAB V4 controller socket.
#[derive(Clone)]
pub struct Socket {
    inner: Arc<SocketInner>,
    handles: Arc<TaskHandles>,
}

impl Socket {
    /// Connects to a relay URL with default options.
    pub async fn connect(url: impl Into<String>) -> Result<Self, WsError> {
        Self::connect_with(SocketOptions::new(url)).await
    }

    /// Connects to a relay with explicit options.
    pub async fn connect_with(options: SocketOptions) -> Result<Self, WsError> {
        Self::connect_with_transport::<TungsteniteTransport>(options).await
    }

    pub(crate) async fn connect_with_transport<T: WsTransport>(
        options: SocketOptions,
    ) -> Result<Self, WsError> {
        if options.url.is_empty() {
            return Err(WsError::InvalidParams("URL must not be empty".to_owned()));
        }

        let (writer_tx, writer_rx) = mpsc::channel(256);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let inner = Arc::new(SocketInner::new(
            writer_tx,
            shutdown_tx,
            options.response_timeout,
        ));

        let transport = T::connect(&options).await?;
        let (sink, source) = transport.split();
        let (hello_tx, hello_rx) = oneshot::channel();
        let reader = tokio::spawn(reader_task(
            Arc::clone(&inner),
            source,
            shutdown_rx.clone(),
            hello_tx,
        ));
        let writer = tokio::spawn(writer_task(
            Arc::clone(&inner),
            sink,
            writer_rx,
            shutdown_rx.clone(),
        ));
        let ping = options.ping_interval.map(|interval| {
            tokio::spawn(ping_task(
                Arc::clone(&inner),
                shutdown_rx,
                interval,
                options.response_timeout,
            ))
        });
        let handles = Arc::new(TaskHandles::new(reader, writer, ping));

        match timeout(options.connect_timeout, hello_rx).await {
            Ok(Ok(())) => Ok(Self { inner, handles }),
            Ok(Err(_)) => {
                inner.fail_transport("hello waiter was dropped".to_owned());
                Err(WsError::NotPaired)
            }
            Err(_) => {
                inner.fail_transport("hello timeout".to_owned());
                Err(WsError::NotPaired)
            }
        }
    }

    /// Returns the controller's target id assigned by the relay.
    #[must_use]
    pub fn target_id(&self) -> &str {
        self.inner.target_id.get().map_or("", String::as_str)
    }

    /// Returns the current connection state.
    #[must_use]
    pub fn state(&self) -> SocketState {
        self.inner.state()
    }

    /// Subscribes to lifecycle and control events.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<SocketEvent> {
        self.inner.events.subscribe()
    }

    /// Waits for the first app client to attach.
    pub async fn wait_for_client(&self) -> Result<AppClient, WsError> {
        let mut shutdown = self.inner.shutdown_tx.subscribe();
        loop {
            if let Some(client) = self
                .inner
                .registry
                .read()
                .expect("registry lock poisoned")
                .first()
            {
                return Ok(client);
            }
            if *shutdown.borrow() || self.state() == SocketState::Closed {
                return Err(WsError::Closed);
            }

            let notify = self
                .inner
                .registry
                .read()
                .expect("registry lock poisoned")
                .notify();
            let notified = notify.notified();
            if let Some(client) = self
                .inner
                .registry
                .read()
                .expect("registry lock poisoned")
                .first()
            {
                return Ok(client);
            }

            tokio::select! {
                () = notified => {}
                _ = shutdown.changed() => return Err(WsError::Closed),
            }
        }
    }

    /// Returns a snapshot of all attached app clients in attach order.
    #[must_use]
    pub fn clients(&self) -> Vec<AppClient> {
        self.inner
            .registry
            .read()
            .expect("registry lock poisoned")
            .clients()
    }

    /// Looks up an attached app client by id.
    #[must_use]
    pub fn client(&self, client_id: &str) -> Option<AppClient> {
        self.inner.app(client_id)
    }

    /// Closes the socket and waits briefly for its tasks to stop.
    pub async fn shutdown(self) -> Result<(), WsError> {
        if self.state() == SocketState::Closed {
            return Ok(());
        }
        self.inner.set_state(SocketState::Closing);
        self.inner.close_clients(&WsError::Closed);
        let _ = timeout(
            Duration::from_secs(1),
            self.inner.writer.send(Outbound::Close {
                code: Some(1000),
                reason: "shutdown".to_owned(),
            }),
        )
        .await;
        let _ = self.inner.shutdown_tx.send(true);
        let handles = self.handles.take();
        let _ = timeout(Duration::from_secs(1), async {
            for handle in handles {
                let _ = handle.await;
            }
        })
        .await;
        self.inner.set_state(SocketState::Closed);
        self.inner.emit(SocketEvent::Closed {
            code: Some(1000),
            reason: "shutdown".to_owned(),
        });
        Ok(())
    }
}

struct TaskHandles {
    handles: Mutex<Vec<JoinHandle<()>>>,
}

impl TaskHandles {
    fn new(reader: JoinHandle<()>, writer: JoinHandle<()>, ping: Option<JoinHandle<()>>) -> Self {
        let mut handles = vec![reader, writer];
        handles.extend(ping);
        Self {
            handles: Mutex::new(handles),
        }
    }

    fn take(&self) -> Vec<JoinHandle<()>> {
        std::mem::take(&mut *self.handles.lock().expect("task handles mutex poisoned"))
    }
}

impl Drop for TaskHandles {
    fn drop(&mut self) {
        for handle in self
            .handles
            .lock()
            .expect("task handles mutex poisoned")
            .drain(..)
        {
            handle.abort();
        }
    }
}

async fn writer_task<S: WsSink>(
    inner: Arc<SocketInner>,
    mut sink: S,
    mut receiver: mpsc::Receiver<Outbound>,
    mut shutdown: watch::Receiver<bool>,
) {
    loop {
        tokio::select! {
            _ = shutdown.changed() => break,
            message = receiver.recv() => {
                let Some(message) = message else { break; };
                let is_close = matches!(message, Outbound::Close { .. });
                if let Err(error) = sink.send(message).await {
                    inner.fail_transport(error.to_string());
                    break;
                }
                if is_close {
                    break;
                }
            }
        }
    }
}

async fn reader_task<S: WsSource>(
    inner: Arc<SocketInner>,
    mut source: S,
    mut shutdown: watch::Receiver<bool>,
    hello: oneshot::Sender<()>,
) {
    let mut hello = Some(hello);
    loop {
        tokio::select! {
            _ = shutdown.changed() => break,
            message = source.recv() => {
                match message {
                    Some(Ok(Inbound::Text(text))) => {
                        inner.handle_text(&text, &mut hello);
                    }
                    Some(Ok(Inbound::Ping(payload))) => {
                        if inner.writer.send(Outbound::Pong(payload)).await.is_err() {
                            inner.fail_transport("writer channel closed".to_owned());
                            break;
                        }
                    }
                    Some(Ok(Inbound::Pong)) => {
                        inner.notify_pong();
                        inner.emit(SocketEvent::Pong { ts: None });
                    }
                    Some(Ok(Inbound::Close { code, reason })) => {
                        inner.close_with(code, reason);
                        break;
                    }
                    Some(Err(error)) => {
                        inner.fail_transport(error.to_string());
                        break;
                    }
                    None => {
                        inner.close_with(None, "websocket stream ended".to_owned());
                        break;
                    }
                }
            }
        }
    }
}

async fn ping_task(
    inner: Arc<SocketInner>,
    mut shutdown: watch::Receiver<bool>,
    interval: Duration,
    response_timeout: Duration,
) {
    let mut ticker = tokio::time::interval(interval);
    loop {
        tokio::select! {
            _ = shutdown.changed() => break,
            _ = ticker.tick() => {
                let (sender, receiver) = oneshot::channel();
                *inner
                    .pong_waiter
                    .lock()
                    .expect("pong waiter mutex poisoned") = Some(sender);
                let frame = match serde_json::to_string(&RelayFrame::Ping) {
                    Ok(frame) => frame,
                    Err(error) => {
                        inner.fail_transport(error.to_string());
                        break;
                    }
                };
                if inner.writer.send(Outbound::Text(frame)).await.is_err() {
                    inner.fail_transport("writer channel closed".to_owned());
                    break;
                }
                if timeout(response_timeout, receiver).await.is_err() {
                    inner.fail_transport("pong timeout".to_owned());
                    break;
                }
            }
        }
    }
}

#[allow(dead_code)]
fn _assert_ws_event_send(_: WsEvent) {}
