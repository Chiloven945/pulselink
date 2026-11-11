//! Device cache and merge logic.

use std::collections::HashMap;

use pulselink_protocol::socket::v4::{DevicePatch, RemoteDevice, SlotPatchEntry};
use serde_json::{Map, Value};

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
    fn from_device(device: RemoteDevice, revision: u64) -> Self {
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

    fn to_device(&self) -> RemoteDevice {
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

/// A cache of the device list for one app client.
#[derive(Debug, Default)]
pub(crate) struct DeviceCache {
    devices: HashMap<String, RemoteDeviceState>,
    order: Vec<String>,
    revision: u64,
}

impl DeviceCache {
    pub(crate) fn snapshot(&self) -> Vec<RemoteDevice> {
        self.order
            .iter()
            .filter_map(|slot_id| self.devices.get(slot_id))
            .map(RemoteDeviceState::to_device)
            .collect()
    }

    pub(crate) fn replace(&mut self, devices: Vec<RemoteDevice>) -> Vec<RemoteDevice> {
        self.devices.clear();
        self.order.clear();
        self.revision = self.revision.saturating_add(1);
        for device in devices {
            let slot_id = device.slot_id.clone();
            self.order.push(slot_id.clone());
            self.devices.insert(
                slot_id,
                RemoteDeviceState::from_device(device, self.revision),
            );
        }
        self.snapshot()
    }

    pub(crate) fn apply_patch(&mut self, patch: &DevicePatch) {
        self.revision = self.revision.saturating_add(1);
        for slot_id in &patch.removed {
            self.devices.remove(slot_id);
            self.order.retain(|existing| existing != slot_id);
        }
        for device in &patch.added {
            let slot_id = device.slot_id.clone();
            if !self.devices.contains_key(&slot_id) {
                self.order.push(slot_id.clone());
            }
            self.devices.insert(
                slot_id,
                RemoteDeviceState::from_device(device.clone(), self.revision),
            );
        }
    }

    pub(crate) fn apply_slots_patch(&mut self, entries: &[SlotPatchEntry]) {
        self.revision = self.revision.saturating_add(1);
        for entry in entries {
            let slot_id = entry.slot_id.clone();
            if !self.devices.contains_key(&slot_id) {
                self.order.push(slot_id.clone());
                self.devices.insert(
                    slot_id.clone(),
                    RemoteDeviceState {
                        id: None,
                        slot_id: slot_id.clone(),
                        name: String::new(),
                        device_type: String::new(),
                        props: Map::new(),
                        slot_state: Map::new(),
                        revision: self.revision,
                    },
                );
            }
            if let Some(device) = self.devices.get_mut(&slot_id) {
                merge_objects(&mut device.props, &entry.props);
                merge_objects(&mut device.slot_state, &entry.slot_state);
                device.revision = self.revision;
            }
        }
    }
}

fn merge_objects(target: &mut Map<String, Value>, patch: &Map<String, Value>) {
    for (key, value) in patch {
        match (target.get_mut(key), value) {
            (Some(Value::Object(target_object)), Value::Object(patch_object)) => {
                merge_objects(target_object, patch_object);
            }
            _ => {
                target.insert(key.clone(), value.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn device(slot: &str, name: &str) -> RemoteDevice {
        RemoteDevice {
            id: None,
            slot_id: slot.to_owned(),
            name: name.to_owned(),
            device_type: "COYOTE_030".to_owned(),
            props: Map::new(),
            slot_state: Map::new(),
        }
    }

    #[test]
    fn snapshot_replaces_everything() {
        let mut cache = DeviceCache::default();
        cache.replace(vec![device("a", "A"), device("b", "B")]);
        cache.replace(vec![device("c", "C")]);
        let snapshot = cache.snapshot();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].slot_id, "c");
    }

    #[test]
    fn patch_adds_and_removes() {
        let mut cache = DeviceCache::default();
        cache.replace(vec![device("a", "A")]);
        cache.apply_patch(&DevicePatch {
            added: vec![device("b", "B")],
            removed: vec!["a".to_owned()],
        });
        let snapshot = cache.snapshot();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].slot_id, "b");
    }

    #[test]
    fn slots_patch_deep_merges_open_fields() {
        let mut cache = DeviceCache::default();
        let mut original = device("a", "A");
        original.props = serde_json::from_value(json!({
            "nested": {"keep": 1, "replace": 1},
            "removeMe": true
        }))
        .expect("object");
        cache.replace(vec![original]);

        cache.apply_slots_patch(&[SlotPatchEntry {
            slot_id: "a".to_owned(),
            props: serde_json::from_value(json!({
                "nested": {"replace": 2, "new": 3},
                "added": "x"
            }))
            .expect("object"),
            slot_state: serde_json::from_value(json!({"hasDevice": true})).expect("object"),
        }]);

        let snapshot = cache.snapshot();
        assert_eq!(snapshot[0].props["nested"]["keep"], 1);
        assert_eq!(snapshot[0].props["nested"]["replace"], 2);
        assert_eq!(snapshot[0].props["nested"]["new"], 3);
        assert_eq!(snapshot[0].props["added"], "x");
        assert_eq!(snapshot[0].slot_state["hasDevice"], true);
    }

    #[test]
    fn slots_patch_creates_placeholder_for_unknown_slot() {
        let mut cache = DeviceCache::default();
        cache.apply_slots_patch(&[SlotPatchEntry {
            slot_id: "new".to_owned(),
            props: serde_json::from_value(json!({"x": 1})).expect("object"),
            slot_state: Map::new(),
        }]);
        let snapshot = cache.snapshot();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].slot_id, "new");
        assert_eq!(snapshot[0].props["x"], 1);
    }

    #[test]
    fn unknown_fields_are_preserved() {
        let mut cache = DeviceCache::default();
        cache.replace(vec![RemoteDevice {
            props: serde_json::from_value(json!({"future": {"a": true}})).expect("object"),
            ..device("a", "A")
        }]);
        let snapshot = cache.snapshot();
        assert_eq!(snapshot[0].props["future"]["a"], true);
    }
}
