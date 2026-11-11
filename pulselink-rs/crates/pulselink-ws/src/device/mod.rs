//! Remote device state, handles, and cache.

mod cache;
mod handle;
mod snapshot;
mod state;

#[cfg(test)]
mod tests;

pub(crate) use cache::DeviceCache;
pub use handle::RemoteDeviceHandle;
pub use snapshot::RemoteDeviceSnapshot;
