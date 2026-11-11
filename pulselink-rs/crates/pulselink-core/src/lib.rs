//! Core domain types for pulselink-rs.
//!
//! This crate is intentionally small and free of I/O. It defines device identity,
//! channels, intensity values, observed state, capabilities, and events. Protocol
//! codecs belong to `pulselink-protocol`; sessions belong to `pulselink-runtime`.
//!
//! The most important rule in this crate is that [`Observed`] distinguishes
//! "unknown" from "known zero". A freshly connected device must not be modelled as
//! having intensity `0` unless the device actually reported that value.

mod capability;
mod channel;
mod device;
mod error;
mod event;
mod id;
mod intensity;
mod state;

pub use capability::{ChannelSet, DeviceCapabilities, IntensityCapabilities, WaveformCapabilities};
pub use channel::{Channel, ChannelMap};
pub use device::{DeviceInfo, DeviceKind};
pub use error::CoreError;
pub use event::{DeviceEvent, IntensitySource};
pub use id::{BleDeviceId, DeviceId, RemoteDeviceId};
pub use intensity::{Intensity, IntensityDelta};
pub use state::{BatteryLevel, BatteryState, ChannelState, ConnectionState, DeviceState, Observed};
