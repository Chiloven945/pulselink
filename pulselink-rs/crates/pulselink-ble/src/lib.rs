//! btleplug-based local BLE backend.
//!
//! This crate implements the `pulselink-transport` traits on top of
//! [`btleplug`]. It does not depend on the protocol or runtime crates and does
//! not expose btleplug types in its public API.

mod error;

#[cfg(feature = "btleplug")]
mod central;
#[cfg(feature = "btleplug")]
mod link;
#[cfg(feature = "btleplug")]
mod notify;
#[cfg(feature = "btleplug")]
mod scan;

pub use error::BleError;

#[cfg(feature = "btleplug")]
pub use central::BtleplugCentral;
#[cfg(feature = "btleplug")]
pub use link::{BtleplugLink, CharacteristicInfo};
