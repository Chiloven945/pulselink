//! Scan result mapping and merge helpers.

use std::collections::{BTreeMap, BTreeSet};

use btleplug::api::PeripheralProperties;
use pulselink_transport::DiscoveredDevice;
use pulselink_transport::ScanFilter;

/// Converts btleplug properties into a transport discovery record.
pub(crate) fn discovered_from_properties(
    id: String,
    properties: Option<&PeripheralProperties>,
) -> DiscoveredDevice {
    let Some(properties) = properties else {
        return DiscoveredDevice {
            id: id.into(),
            name: None,
            rssi: None,
            service_uuids: Vec::new(),
            manufacturer_data: Vec::new(),
        };
    };

    let name = properties
        .local_name
        .clone()
        .or_else(|| properties.advertisement_name.clone());
    let mut service_uuids: Vec<_> = properties.services.clone();
    service_uuids.sort_unstable();
    service_uuids.dedup();

    let mut manufacturer_data: Vec<_> = properties
        .manufacturer_data
        .iter()
        .map(|(id, data)| (*id, data.clone()))
        .collect();
    manufacturer_data.sort_unstable_by_key(|(id, _)| *id);

    DiscoveredDevice {
        id: id.into(),
        name,
        rssi: properties.rssi,
        service_uuids,
        manufacturer_data,
    }
}

/// Merges a fresh observation into the scan accumulator.
///
/// Newer observations win for the name and RSSI, while service UUIDs and
/// manufacturer data are accumulated. Keeping the union makes the result stable
/// when the final `peripherals()` sweep contains older advertisement data.
pub(crate) fn merge_discovered(
    devices: &mut BTreeMap<String, DiscoveredDevice>,
    update: DiscoveredDevice,
) {
    let key = update.id.to_string();
    if let Some(existing) = devices.get_mut(&key) {
        if update.name.is_some() {
            existing.name = update.name;
        }
        if update.rssi.is_some() {
            existing.rssi = update.rssi;
        }

        let mut services: BTreeSet<_> = existing.service_uuids.iter().copied().collect();
        services.extend(update.service_uuids);
        existing.service_uuids = services.into_iter().collect();

        let mut manufacturer: BTreeMap<_, _> = existing.manufacturer_data.iter().cloned().collect();
        manufacturer.extend(update.manufacturer_data);
        existing.manufacturer_data = manufacturer.into_iter().collect();
    } else {
        devices.insert(key, update);
    }
}

/// Applies the SDK-level filter to a scan accumulator and returns sorted results.
pub(crate) fn finish_scan(
    devices: BTreeMap<String, DiscoveredDevice>,
    filter: &ScanFilter,
) -> Vec<DiscoveredDevice> {
    devices
        .into_values()
        .filter(|device| filter.matches(device))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn device(name: &str, rssi: i16) -> DiscoveredDevice {
        DiscoveredDevice {
            id: "dev-1".into(),
            name: Some(name.to_owned()),
            rssi: Some(rssi),
            service_uuids: Vec::new(),
            manufacturer_data: Vec::new(),
        }
    }

    #[test]
    fn newer_name_and_rssi_win() {
        let mut devices = BTreeMap::new();
        merge_discovered(&mut devices, device("Old", -80));
        merge_discovered(&mut devices, device("New", -40));

        let merged = devices.get("dev-1").expect("device");
        assert_eq!(merged.name.as_deref(), Some("New"));
        assert_eq!(merged.rssi, Some(-40));
    }

    #[test]
    fn missing_fields_do_not_erase_existing_values() {
        let mut devices = BTreeMap::new();
        let mut first = device("Coyote", -70);
        first.service_uuids.push(Uuid::from_u128(0x180c));
        merge_discovered(&mut devices, first);

        let mut update = device("Coyote", -70);
        update.name = None;
        update.rssi = None;
        update.service_uuids.push(Uuid::from_u128(0x180a));
        merge_discovered(&mut devices, update);

        let merged = devices.get("dev-1").expect("device");
        assert_eq!(merged.name.as_deref(), Some("Coyote"));
        assert_eq!(merged.rssi, Some(-70));
        assert_eq!(merged.service_uuids.len(), 2);
    }

    #[test]
    fn manufacturer_data_is_merged_by_key() {
        let mut devices = BTreeMap::new();
        let mut first = device("Coyote", -70);
        first.manufacturer_data.push((1, vec![1]));
        merge_discovered(&mut devices, first);

        let mut update = device("Coyote", -70);
        update.manufacturer_data.push((2, vec![2]));
        merge_discovered(&mut devices, update);

        let merged = devices.get("dev-1").expect("device");
        assert_eq!(merged.manufacturer_data, vec![(1, vec![1]), (2, vec![2])]);
    }

    #[test]
    fn finish_scan_applies_sdk_filter() {
        let mut devices = BTreeMap::new();
        merge_discovered(&mut devices, device("47L121000", -40));
        let filter = ScanFilter {
            name_prefixes: vec!["47L".to_owned()],
            ..ScanFilter::default()
        };
        let result = finish_scan(devices, &filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name.as_deref(), Some("47L121000"));
    }
}
