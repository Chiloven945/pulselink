//! Interactive terminal test bench for all supported devices.
//!
//! The crate is intentionally not published. It exposes its command, model, UI,
//! and worker modules so integration tests can drive the same code used by the
//! terminal application.

pub mod app;
pub mod command;
pub mod model;
pub mod ui;
pub mod worker;
