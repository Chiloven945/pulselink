//! Sans-io DG-LAB protocol codecs.
//!
//! The crate contains two feature-gated protocol families:
//!
//! - `coyote-v3` (default): Coyote V3 BLE binary codecs;
//! - `socket-v4`: DG-LAB V4 WebSocket JSON types.
//!
//! It does not perform I/O and does not depend on Tokio or btleplug.

pub mod error;

#[cfg(feature = "coyote-v3")]
pub mod coyote;

#[cfg(feature = "socket-v4")]
pub mod socket;

pub use error::ProtocolError;
