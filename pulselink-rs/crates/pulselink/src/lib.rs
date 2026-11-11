//! `pulselink` facade.
//!
//! This crate is the single entry point for most users. The `ble` feature is
//! enabled by default; `websocket` enables the DG-LAB V4 controller SDK and
//! `protocol` enables both sans-io protocol families without a transport.

#[cfg(feature = "ble")]
mod client;
mod error;

pub use error::Error;

#[cfg(feature = "ble")]
pub use client::PulseLink;

pub mod pattern {
    //! Waveform timeline types.
    pub use pulselink_pattern::*;
}

#[cfg(any(feature = "ble", feature = "websocket", feature = "protocol"))]
pub mod protocol {
    //! Sans-io DG-LAB protocol codecs.
    pub use pulselink_protocol::*;
}

pub mod transport {
    //! Local device transport SPI.
    pub use pulselink_transport::*;
}

/// btleplug BLE backend.
#[cfg(feature = "ble")]
pub mod ble {
    pub use pulselink_ble::*;
}

/// Coyote V3 session runtime.
#[cfg(feature = "ble")]
pub mod runtime {
    pub use pulselink_runtime::*;
}

/// DG-LAB V4 WebSocket session API.
#[cfg(feature = "websocket")]
pub mod ws {
    pub use pulselink_ws::*;
}

pub use pulselink_core::{
    BatteryLevel, BatteryState, BleDeviceId, Channel, ChannelMap, ChannelSet, ChannelState,
    ConnectionState, CoreError, DeviceCapabilities, DeviceEvent, DeviceId, DeviceInfo, DeviceKind,
    DeviceState, Intensity, IntensityCapabilities, IntensityDelta, IntensitySource, Observed,
    RemoteDeviceId, WaveformCapabilities,
};
