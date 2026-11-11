//! Local device transport SPI.
//!
//! This crate defines the port used by `pulselink-runtime`. It has no Tokio or
//! btleplug dependency. Platform backends such as `pulselink-ble` implement the
//! traits in this crate.

mod central;
mod error;
mod link;

#[cfg(feature = "mock")]
pub mod mock;

pub use central::{Backend, Central, Connector, ReconnectTarget};
pub use error::TransportError;
pub use link::{
    DiscoveredDevice, GattLink, NotificationStream, ScanFilter, TransportDeviceId, WriteMode,
};
