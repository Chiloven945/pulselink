//! Socket reader, writer, ping, and task lifetime management.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use pulselink_protocol::socket::v4::RelayFrame;
use tokio::sync::{mpsc, oneshot, watch};
use tokio::task::JoinHandle;
use tokio::time::timeout;

use crate::app::WsEvent;
use crate::transport::{Inbound, Outbound, WsSink, WsSource};

use super::inner::SocketInner;
use super::state::SocketEvent;

pub(super) struct TaskHandles {
    handles: Mutex<Vec<JoinHandle<()>>>,
}

impl TaskHandles {
    pub(super) fn new(
        reader: JoinHandle<()>,
        writer: JoinHandle<()>,
        ping: Option<JoinHandle<()>>,
    ) -> Self {
        let mut handles = vec![reader, writer];
        handles.extend(ping);
        Self {
            handles: Mutex::new(handles),
        }
    }

    pub(super) fn take(&self) -> Vec<JoinHandle<()>> {
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

pub(super) async fn writer_task<S: WsSink>(
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

pub(super) async fn reader_task<S: WsSource>(
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

pub(super) async fn ping_task(
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
