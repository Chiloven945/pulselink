//! Device session runtime.
//!
//! Implementation starts in milestone M6. This crate will contain the Session
//! Actor, Sequence/ACK manager, 100 ms scheduler, safety policy, and reconnect
//! logic. It depends on `pulselink-transport`, not on btleplug.

#![doc = "See `docs/ROADMAP.md` for the runtime contracts."]
