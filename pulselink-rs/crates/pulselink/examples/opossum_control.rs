//! Scan and exercise an Opossum vibration controller.

use std::time::Duration;

use pulselink::device::OPOSSUM;
use pulselink::{Channel, Intensity, LocalSession, PulseLink};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let link = PulseLink::new().await?;
    let devices = link.scan_profile(&OPOSSUM, Duration::from_secs(5)).await?;
    let device = devices.first().ok_or("no Opossum found")?;
    let session = link.connect_device(device).await?;
    let LocalSession::Opossum(session) = session else {
        return Err("device did not classify as Opossum".into());
    };

    session.set_pulse(Channel::A, [20, 20, 20, 20]).await?;
    session
        .set_intensity(Channel::A, Intensity::new(5)?)
        .await?;
    tokio::time::sleep(Duration::from_secs(2)).await;
    session.shutdown().await?;
    Ok(())
}
