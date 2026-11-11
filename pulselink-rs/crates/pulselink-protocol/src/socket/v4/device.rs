//! DG-LAB V4 device descriptors and incremental patches.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A device exposed by a DG-LAB V4 app client.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RemoteDevice {
    /// Optional device index.
    #[serde(default)]
    pub id: Option<u32>,
    /// Device slot id used by device operations.
    #[serde(rename = "slotId")]
    pub slot_id: String,
    /// Display name.
    pub name: String,
    /// Device type, such as `COYOTE_030`.
    #[serde(rename = "type")]
    pub device_type: String,
    /// Open-ended device properties.
    #[serde(default)]
    pub props: Map<String, Value>,
    /// Open-ended device slot state.
    #[serde(default, rename = "slotState")]
    pub slot_state: Map<String, Value>,
}

/// Incremental device list patch.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DevicePatch {
    /// Added or replaced devices.
    #[serde(default)]
    pub added: Vec<RemoteDevice>,
    /// Removed slot ids.
    #[serde(default)]
    pub removed: Vec<String>,
}

/// Incremental slot state patch.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SlotPatch {
    /// Changed slots.
    #[serde(default)]
    pub slots: Vec<SlotPatchEntry>,
}

/// One changed slot in a `slots.patch` event.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SlotPatchEntry {
    /// Device slot id.
    #[serde(rename = "slotId")]
    pub slot_id: String,
    /// Changed properties.
    #[serde(default)]
    pub props: Map<String, Value>,
    /// Changed slot state.
    #[serde(default, rename = "slotState")]
    pub slot_state: Map<String, Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_remote_device() {
        let json = r#"{
            "id": 0,
            "slotId": "slot-a",
            "name": "Device A",
            "type": "COYOTE_030",
            "props": {"power": 80, "intensityA": 10},
            "slotState": {"markLight": "yellow", "hasDevice": true}
        }"#;

        let device: RemoteDevice = serde_json::from_str(json).expect("valid device");
        assert_eq!(device.id, Some(0));
        assert_eq!(device.slot_id, "slot-a");
        assert_eq!(device.device_type, "COYOTE_030");
        assert_eq!(device.props["power"], 80);
        assert_eq!(device.slot_state["hasDevice"], true);
    }

    #[test]
    fn preserves_unknown_props() {
        let json = r#"{
            "slotId": "slot-a",
            "name": "Device A",
            "type": "COYOTE_030",
            "props": {"futureField": {"nested": true}},
            "slotState": {"newState": 1}
        }"#;

        let device: RemoteDevice = serde_json::from_str(json).expect("valid device");
        assert!(device.props.contains_key("futureField"));
        assert!(device.slot_state.contains_key("newState"));
    }

    #[test]
    fn parses_patches() {
        let patch: DevicePatch = serde_json::from_str(
            r#"{"added":[{"slotId":"slot-b","name":"B","type":"COYOTE_030"}],"removed":["slot-a"]}"#,
        )
        .expect("valid device patch");
        assert_eq!(patch.added.len(), 1);
        assert_eq!(patch.removed, vec!["slot-a"]);

        let slot_patch: SlotPatch = serde_json::from_str(
            r#"{"slots":[{"slotId":"slot-a","props":{"strength":10},"slotState":{"markLight":"green"}}]}"#,
        )
        .expect("valid slot patch");
        assert_eq!(slot_patch.slots.len(), 1);
        assert_eq!(slot_patch.slots[0].props["strength"], 10);
    }
}
