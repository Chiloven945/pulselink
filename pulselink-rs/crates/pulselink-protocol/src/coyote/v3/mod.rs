//! Coyote V3 protocol codecs.

pub mod b0;
pub mod b1;
pub mod battery;
pub mod bf;
pub mod frequency;
pub mod gatt;

pub use b0::{
    B0Command, ChannelWaveform, IntensityOp, Sequence, StereoIntensityOp, StereoWaveform,
};
pub use b1::B1Message;
pub use battery::{BatteryLevel, decode_battery};
pub use bf::BfCommand;
pub use frequency::{decode_wire_frequency, encode_interval};
