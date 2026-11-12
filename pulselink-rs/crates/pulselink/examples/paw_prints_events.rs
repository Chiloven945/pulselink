//! Scan and print `PawPrints` events.

use std::time::Duration;

use pulselink::device::PAW_PRINTS;
use pulselink::protocol::paw_prints::Command50;
use pulselink::{LocalSession, PulseLink};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let link = PulseLink::new().await?;
    let devices = link
        .scan_profile(&PAW_PRINTS, Duration::from_secs(5))
        .await?;
    let device = devices.first().ok_or("no PawPrints found")?;
    let session = link.connect_device(device).await?;
    let LocalSession::PawPrints(session) = session else {
        return Err("device did not classify as PawPrints".into());
    };
    session.configure(Command50::no_trigger(1)).await?;

    let mut events = session.subscribe();
    for _ in 0..10 {
        if let Ok(event) = tokio::time::timeout(Duration::from_millis(500), events.recv()).await {
            println!("{event:?}");
        }
    }
    session.shutdown().await?;
    Ok(())
}
