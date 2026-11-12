//! Print typed snapshots of devices exposed by a DG-LAB V4 app.

use pulselink::ws::{AppClient, Socket, TypedRemoteSnapshot};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url =
        std::env::var("PULSELINK_WS_URL").unwrap_or_else(|_| "ws://127.0.0.1:8080".to_owned());
    let socket = Socket::connect(url).await?;
    println!("pair target id: {}", socket.target_id());
    let client: AppClient = socket.wait_for_client().await?;
    let devices = client.request_devices().await?;
    for device in devices {
        let handle = client.device(&device.slot_id).ok_or("device disappeared")?;
        match handle.typed_snapshot() {
            TypedRemoteSnapshot::Coyote(snapshot) => {
                println!(
                    "coyote {:?}: {:?}",
                    snapshot.snapshot().slot_id,
                    snapshot.intensity_a()
                );
            }
            TypedRemoteSnapshot::Opossum(snapshot) => {
                println!(
                    "opossum {:?}: {:?}",
                    snapshot.snapshot().slot_id,
                    snapshot.mode()
                );
            }
            TypedRemoteSnapshot::Civet(snapshot) => {
                println!(
                    "civet {:?}: {:?}",
                    snapshot.snapshot().slot_id,
                    snapshot.pressure()
                );
            }
            TypedRemoteSnapshot::Unknown(snapshot) => {
                println!("unknown {}: {}", snapshot.slot_id, snapshot.device_type);
            }
        }
    }
    socket.shutdown().await?;
    Ok(())
}
