//! Public remote-device snapshots and typed accessors.

use serde_json::{Map, Value};

use super::state::RemoteDeviceState;

/// A snapshot of one remote device.
#[derive(Clone, Debug, PartialEq)]
pub struct RemoteDeviceSnapshot {
    /// Optional device index.
    pub id: Option<u32>,
    /// Device slot id.
    pub slot_id: String,
    /// Display name.
    pub name: String,
    /// Device type.
    pub device_type: String,
    /// Open-ended device properties.
    pub props: Map<String, Value>,
    /// Open-ended slot state.
    pub slot_state: Map<String, Value>,
    /// Cache revision.
    pub revision: u64,
}

impl RemoteDeviceSnapshot {
    pub(super) fn from_state(state: &RemoteDeviceState) -> Self {
        Self {
            id: state.id,
            slot_id: state.slot_id.clone(),
            name: state.name.clone(),
            device_type: state.device_type.clone(),
            props: state.props.clone(),
            slot_state: state.slot_state.clone(),
            revision: state.revision,
        }
    }

    /// Returns `props.intensityA` as an integer.
    #[must_use]
    pub fn intensity_a(&self) -> Option<u8> {
        json_u8(self.props.get("intensityA"))
    }

    /// Returns `props.intensityB` as an integer.
    #[must_use]
    pub fn intensity_b(&self) -> Option<u8> {
        json_u8(self.props.get("intensityB"))
    }

    /// Returns the battery/power value from the app's open props.
    #[must_use]
    pub fn battery(&self) -> Option<u8> {
        json_u8(self.props.get("power")).or_else(|| json_u8(self.props.get("battery")))
    }

    /// Returns `slotState.hasDevice`.
    #[must_use]
    pub fn has_device(&self) -> Option<bool> {
        self.slot_state.get("hasDevice").and_then(Value::as_bool)
    }

    /// Returns `slotState.markLight`.
    #[must_use]
    pub fn mark_light(&self) -> Option<&str> {
        self.slot_state.get("markLight").and_then(Value::as_str)
    }
}

fn json_u8(value: Option<&Value>) -> Option<u8> {
    let value = value?;
    value
        .as_u64()
        .and_then(|value| u8::try_from(value).ok())
        .or_else(|| value.as_i64().and_then(|value| u8::try_from(value).ok()))
        .or_else(|| value.as_str().and_then(|value| value.parse().ok()))
}
