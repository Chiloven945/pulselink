//! `pulselink` facade.
//!
//! This crate is the single entry point for most users. The `ble` feature is
//! enabled by default; `websocket` enables the DG-LAB V4 controller SDK.
//!
//! Implementation of the high-level `PulseLink` client starts in milestone M8.
//! The modules below are already exposed so advanced users can depend on the
//! individual crates directly.

pub mod pattern {
    //! Waveform timeline types.
    pub use pulselink_pattern::*;
}

#[cfg(any(feature = "ble", feature = "websocket"))]
pub mod protocol {
    //! Sans-io DG-LAB protocol codecs.
    pub use pulselink_protocol::*;
}

pub mod transport {
    //! Local device transport SPI.
    pub use pulselink_transport::*;
}

// The following modules are added when the corresponding crates expose their
// first public items:
//
// - `ble` (M4, feature `ble`)
// - `runtime` (M6, feature `ble`)
// - `ws` (M5/M7, feature `websocket`)

pub use pulselink_core::{
    BatteryLevel, BatteryState, BleDeviceId, Channel, ChannelMap, ChannelSet, ChannelState,
    ConnectionState, CoreError, DeviceCapabilities, DeviceEvent, DeviceId, DeviceInfo, DeviceKind,
    DeviceState, Intensity, IntensityCapabilities, IntensityDelta, IntensitySource, Observed,
    RemoteDeviceId, WaveformCapabilities,
};
