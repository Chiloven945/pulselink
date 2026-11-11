//! Shared remote-device state.

use std::sync::{Arc, RwLock};

use pulselink_protocol::socket::v4::RemoteDevice;
use serde_json::{Map, Value};

pub(crate) type SharedState = Arc<RwLock<RemoteDeviceState>>;

/// Internal device state with a revision.
#[derive(Clone, Debug)]
pub(crate) struct RemoteDeviceState {
    pub(crate) id: Option<u32>,
    pub(crate) slot_id: String,
    pub(crate) name: String,
    pub(crate) device_type: String,
    pub(crate) props: Map<String, Value>,
    pub(crate) slot_state: Map<String, Value>,
    pub(crate) revision: u64,
}

impl RemoteDeviceState {
    pub(super) fn from_device(device: RemoteDevice, revision: u64) -> Self {
        Self {
            id: device.id,
            slot_id: device.slot_id,
            name: device.name,
            device_type: device.device_type,
            props: device.props,
            slot_state: device.slot_state,
            revision,
        }
    }

    pub(super) fn to_device(&self) -> RemoteDevice {
        RemoteDevice {
            id: self.id,
            slot_id: self.slot_id.clone(),
            name: self.name.clone(),
            device_type: self.device_type.clone(),
            props: self.props.clone(),
            slot_state: self.slot_state.clone(),
        }
    }
}
