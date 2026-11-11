//! Socket state and lifecycle events.

/// Connection state of a [`crate::Socket`].
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

/// A lifecycle or control event emitted by a [`crate::Socket`].
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
