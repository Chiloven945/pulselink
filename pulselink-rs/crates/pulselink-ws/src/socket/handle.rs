//! Public controller socket handle.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{broadcast, mpsc, oneshot, watch};
use tokio::time::timeout;

use crate::app::AppClient;
use crate::error::WsError;
use crate::options::SocketOptions;
use crate::transport::{Outbound, TungsteniteTransport, WsTransport};

use super::inner::SocketInner;
use super::state::{SocketEvent, SocketState};
use super::tasks::{TaskHandles, ping_task, reader_task, writer_task};

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
