//! Minimal safe Coyote V3 control example.
//!
//! The example refuses to connect unless `PULSELINK_HW=1` is set. It applies a
//! very small intensity, stops immediately, and always shuts the session down.

use std::time::Duration;

use pulselink::protocol::coyote::v3::gatt::{DEVICE_NAME, SERVICE_CONTROL};
use pulselink::transport::ScanFilter;
use pulselink::{Channel, Intensity, PulseLink};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("PULSELINK_HW").as_deref() != Ok("1") {
        println!("set PULSELINK_HW=1 to run the BLE control example");
        return Ok(());
    }

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
    let device = devices.first().ok_or("no Coyote V3 device found")?;

    let session = link.connect_coyote_v3(device).await?;
    println!("connected: {:?}", session.state().common.connection);

    session
        .set_intensity(Channel::A, Intensity::new(1)?)
        .await?;
    tokio::time::sleep(Duration::from_millis(500)).await;
    session.stop().await?;
    session.shutdown().await?;
    link.shutdown().await;
    Ok(())
}
