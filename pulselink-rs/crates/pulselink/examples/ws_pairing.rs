//! Connect to a V4 relay and display the pairing target id.

use std::time::Duration;

use pulselink::ws::Socket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "wss://trex.dungeon-lab.cn/v4".to_owned());
    let socket = Socket::connect(&url).await?;
    println!("target id: {}", socket.target_id());
    println!("waiting for a DG-LAB app...");

    let client = tokio::time::timeout(Duration::from_secs(300), socket.wait_for_client()).await??;
    println!("app attached: {}", client.client_id());
    println!("app ping: {}", client.ping().await?);

    socket.shutdown().await?;
    Ok(())
}
