use pulselink_core::DeviceKind;
use pulselink_protocol::socket::v4::{DevicePatch, RemoteDevice, SlotPatchEntry};
use serde_json::{Map, json};

use super::cache::DeviceCache;
use super::typed::TypedRemoteSnapshot;

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

#[test]
fn snapshot_typed_accessors() {
    let mut cache = DeviceCache::default();
    cache.replace(vec![RemoteDevice {
        props: serde_json::from_value(json!({
            "intensityA": 12,
            "intensityB": "34",
            "power": 88
        }))
        .expect("object"),
        slot_state: serde_json::from_value(json!({
            "hasDevice": true,
            "markLight": "green"
        }))
        .expect("object"),
        ..device("a", "A")
    }]);
    let snapshot = cache.snapshot_device("a").expect("snapshot");
    assert_eq!(snapshot.intensity_a(), Some(12));
    assert_eq!(snapshot.intensity_b(), Some(34));
    assert_eq!(snapshot.battery(), Some(88));
    assert_eq!(snapshot.has_device(), Some(true));
    assert_eq!(snapshot.mark_light(), Some("green"));
}

#[test]
fn typed_snapshot_maps_coyote_and_opossum() {
    let mut cache = DeviceCache::default();
    let mut coyote = device("coyote", "Coyote");
    coyote.device_type = "COYOTE_030".to_owned();
    coyote.props = serde_json::from_value(json!({
        "intensityA": 12,
        "intensityB": 34,
        "channelAStatus": 1,
        "channelBStatus": 0
    }))
    .expect("object");
    let mut opossum = device("opossum", "Opossum");
    opossum.device_type = "OVC_1".to_owned();
    opossum.props = serde_json::from_value(json!({
        "intensityA": 5,
        "intensityB": 6,
        "mode": 2,
        "channelAStatus": true,
        "channelBStatus": false
    }))
    .expect("object");
    cache.replace(vec![coyote, opossum]);

    let coyote = cache.snapshot_device("coyote").expect("coyote").typed();
    assert!(matches!(coyote, TypedRemoteSnapshot::Coyote(_)));
    let TypedRemoteSnapshot::Coyote(coyote) = coyote else {
        unreachable!()
    };
    assert_eq!(coyote.snapshot().kind(), DeviceKind::CoyoteV3);
    assert_eq!(coyote.intensity_a(), Some(12));
    assert_eq!(coyote.channel_b_status(), Some(0));

    let opossum = cache.snapshot_device("opossum").expect("opossum").typed();
    let TypedRemoteSnapshot::Opossum(opossum) = opossum else {
        panic!("expected Opossum");
    };
    assert_eq!(opossum.mode(), Some(2));
    assert_eq!(opossum.channel_a_connected(), Some(true));
    assert_eq!(opossum.channel_b_connected(), Some(false));
}

#[test]
fn typed_snapshot_maps_civet() {
    let mut cache = DeviceCache::default();
    let mut civet = device("civet", "Civet");
    civet.device_type = "BMTR_1".to_owned();
    civet.props = serde_json::from_value(json!({"pressure": 7.9})).expect("object");
    cache.replace(vec![civet]);
    let snapshot = cache.snapshot_device("civet").expect("civet").typed();
    let TypedRemoteSnapshot::Civet(civet) = snapshot else {
        panic!("expected Civet");
    };
    assert_eq!(
        civet.pressure().map(pulselink_core::Pressure::as_centi_kpa),
        Some(790)
    );
}
