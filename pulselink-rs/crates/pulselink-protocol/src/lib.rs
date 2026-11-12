//! Sans-io DG-LAB protocol codecs.
//!
//! The crate contains two feature-gated protocol families:
//!
//! - `coyote-v2` / `coyote-v3`: Coyote BLE binary codecs;
//! - `opossum`, `civet`, `paw-prints`: other DG-LAB BLE devices;
//! - `socket-v4`: DG-LAB V4 WebSocket JSON types.
//!
//! It does not perform I/O and does not depend on Tokio or btleplug.

pub mod error;

#[cfg(any(feature = "coyote-v2", feature = "coyote-v3"))]
pub mod coyote;

#[cfg(feature = "civet")]
pub mod civet;

#[cfg(feature = "opossum")]
pub mod opossum;

#[cfg(feature = "paw-prints")]
pub mod paw_prints;

#[cfg(feature = "socket-v4")]
pub mod socket;

pub use error::ProtocolError;
