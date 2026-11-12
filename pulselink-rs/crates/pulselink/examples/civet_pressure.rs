//! Scan and print Civet pressure readings.

use std::time::Duration;

use pulselink::device::CIVET;
use pulselink::{LocalSession, PulseLink};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let link = PulseLink::new().await?;
    let devices = link.scan_profile(&CIVET, Duration::from_secs(5)).await?;
    let device = devices.first().ok_or("no Civet found")?;
    let session = link.connect_device(device).await?;
    let LocalSession::Civet(session) = session else {
        return Err("device did not classify as Civet".into());
    };

    for _ in 0..5 {
        tokio::time::sleep(Duration::from_millis(200)).await;
        println!("pressure: {:?}", session.pressure());
    }
    session.shutdown().await?;
    Ok(())
}
