//! Device cache and open-field merge logic.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use pulselink_protocol::socket::v4::{DevicePatch, RemoteDevice, SlotPatchEntry};
use serde_json::{Map, Value};

use super::snapshot::RemoteDeviceSnapshot;
use super::state::{RemoteDeviceState, SharedState};

/// A cache of the device list for one app client.
#[derive(Debug, Default)]
pub(crate) struct DeviceCache {
    devices: HashMap<String, SharedState>,
    order: Vec<String>,
    revision: u64,
}

impl DeviceCache {
    pub(crate) fn snapshot(&self) -> Vec<RemoteDevice> {
        self.order
            .iter()
            .filter_map(|slot_id| self.devices.get(slot_id))
            .map(|state| {
                state
                    .read()
                    .expect("device state lock poisoned")
                    .to_device()
            })
            .collect()
    }

    pub(crate) fn shared(&self, slot_id: &str) -> Option<SharedState> {
        self.devices.get(slot_id).cloned()
    }

    pub(crate) fn snapshot_device(&self, slot_id: &str) -> Option<RemoteDeviceSnapshot> {
        self.devices.get(slot_id).map(|state| {
            RemoteDeviceSnapshot::from_state(&state.read().expect("device state lock poisoned"))
        })
    }

    pub(crate) fn replace(&mut self, devices: Vec<RemoteDevice>) -> Vec<RemoteDevice> {
        self.revision = self.revision.saturating_add(1);
        let revision = self.revision;
        let mut next = HashMap::new();
        self.order.clear();
        for device in devices {
            let slot_id = device.slot_id.clone();
            let state = RemoteDeviceState::from_device(device, revision);
            let shared = if let Some(existing) = self.devices.get(&slot_id) {
                *existing.write().expect("device state lock poisoned") = state;
                Arc::clone(existing)
            } else {
                Arc::new(RwLock::new(state))
            };
            self.order.push(slot_id.clone());
            next.insert(slot_id, shared);
        }
        self.devices = next;
        self.snapshot()
    }

    pub(crate) fn apply_patch(&mut self, patch: &DevicePatch) {
        self.revision = self.revision.saturating_add(1);
        let revision = self.revision;
        for slot_id in &patch.removed {
            self.devices.remove(slot_id);
            self.order.retain(|existing| existing != slot_id);
        }
        for device in &patch.added {
            let slot_id = device.slot_id.clone();
            let state = RemoteDeviceState::from_device(device.clone(), revision);
            if let Some(existing) = self.devices.get(&slot_id) {
                *existing.write().expect("device state lock poisoned") = state;
            } else {
                self.order.push(slot_id.clone());
                self.devices.insert(slot_id, Arc::new(RwLock::new(state)));
            }
        }
    }

    pub(crate) fn apply_slots_patch(&mut self, entries: &[SlotPatchEntry]) {
        self.revision = self.revision.saturating_add(1);
        let revision = self.revision;
        for entry in entries {
            let slot_id = entry.slot_id.clone();
            let shared = if let Some(existing) = self.devices.get(&slot_id) {
                Arc::clone(existing)
            } else {
                let state = RemoteDeviceState {
                    id: None,
                    slot_id: slot_id.clone(),
                    name: String::new(),
                    device_type: String::new(),
                    props: Map::new(),
                    slot_state: Map::new(),
                    revision,
                };
                let shared = Arc::new(RwLock::new(state));
                self.order.push(slot_id.clone());
                self.devices.insert(slot_id.clone(), Arc::clone(&shared));
                shared
            };
            let mut state = shared.write().expect("device state lock poisoned");
            merge_objects(&mut state.props, &entry.props);
            merge_objects(&mut state.slot_state, &entry.slot_state);
            state.revision = revision;
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
