//! List V4 devices and exercise safe intensity/reset operations.

use std::time::Duration;

use pulselink::ws::{Channel, ClearTarget, Intensity, OperationOptions, Socket};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "wss://trex.dungeon-lab.cn/v4".to_owned());
    let socket = Socket::connect(&url).await?;
    println!("target id: {}", socket.target_id());

    let client = tokio::time::timeout(Duration::from_secs(300), socket.wait_for_client()).await??;
    let devices = client.request_devices().await?;
    let device = devices.first().ok_or("no device is available in the app")?;
    let handle = client
        .device(&device.slot_id)
        .ok_or("device handle is not available")?;
    println!("device: {} ({})", handle.name(), handle.device_type());

    let options = OperationOptions::default().with_timeout(Duration::from_secs(5));
    handle
        .reset_intensity(Channel::A, options)
        .await?
        .completion()
        .await?;
    handle
        .set_temp_intensity(
            Channel::A,
            Intensity::new(5)?,
            Duration::from_millis(500),
            options,
        )
        .await?
        .completion()
        .await?;

    client.clear_operate(ClearTarget::All).await?;
    socket.shutdown().await?;
    Ok(())
}
