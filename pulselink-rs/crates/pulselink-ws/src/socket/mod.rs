//! Controller socket, reader/writer tasks, and app routing.

mod handle;
mod inner;
mod state;
mod tasks;

pub use handle::Socket;
pub use state::{SocketEvent, SocketState};
