//! Scan for nearby Coyote V3 devices.

use std::time::Duration;

use pulselink::PulseLink;
use pulselink::protocol::coyote::v3::gatt::{DEVICE_NAME, SERVICE_CONTROL};
use pulselink::transport::ScanFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let link = PulseLink::new().await?;
    let devices = link
        .scan(
            ScanFilter {
                name_prefixes: vec![DEVICE_NAME.to_owned()],
                service_uuids: vec![SERVICE_CONTROL],
            },
            Duration::from_secs(5),
        )
        .await?;

    for device in &devices {
        println!(
            "{:?} name={:?} rssi={:?} services={}",
            device.id,
            device.name,
            device.rssi,
            device.service_uuids.len()
        );
    }
    if devices.is_empty() {
        println!("no Coyote V3 device found");
    }

    link.shutdown().await;
    Ok(())
}
