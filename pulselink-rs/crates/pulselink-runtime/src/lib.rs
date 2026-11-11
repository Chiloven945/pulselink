//! Device session runtime for `pulselink-rs`.
//!
//! The runtime owns the Coyote V3 session actor, sequence/ACK handling, the
//! 100 ms scheduler, safety policy, reconnection, and shutdown. It depends on
//! `pulselink-transport`, not on btleplug.

mod client;
mod config;
mod error;
mod event;
mod safety;
mod scheduler;
mod sequence;
mod session;
mod state;

pub use client::Client;
pub use config::{ClientConfig, CoyoteV3Balance, CoyoteV3Config, ReconnectPolicy, SafetyPolicy};
pub use error::RuntimeError;
pub use event::CoyoteV3Event;
pub use session::{CoyoteV3Session, PlayHandle};
pub use state::{
    CoyoteV3Settings, CoyoteV3State, IntensityChange, PatternId, PatternPlaybackState,
    PlayEndReason, PlayOptions, PlaybackState, StopOutcome,
};
