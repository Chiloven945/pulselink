//! Scan and control a Coyote V2 with a deliberately low intensity.

use std::time::Duration;

use pulselink::device::COYOTE_V2;
use pulselink::protocol::coyote::v2::CoyoteV2Intensity;
use pulselink::{Channel, LocalSession, PulseLink};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let link = PulseLink::new().await?;
    let devices = link
        .scan_profile(&COYOTE_V2, Duration::from_secs(5))
        .await?;
    let device = devices.first().ok_or("no Coyote V2 found")?;
    let session = link.connect_device(device).await?;
    let LocalSession::CoyoteV2(session) = session else {
        return Err("device did not classify as Coyote V2".into());
    };

    let intensity = CoyoteV2Intensity::from_display(5)?;
    session.set_intensity(Channel::A, intensity).await?;
    tokio::time::sleep(Duration::from_secs(2)).await;
    session
        .set_intensity(Channel::A, CoyoteV2Intensity::MIN)
        .await?;
    session.shutdown().await?;
    Ok(())
}
