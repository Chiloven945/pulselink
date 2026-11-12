//! Standalone local relay binary.

use std::net::SocketAddr;
use std::time::Duration;

use pulselink_relay::{LocalRelay, RelayConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = RelayConfig {
        idle_timeout: Some(Duration::from_secs(300)),
        ..RelayConfig::default()
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bind" => {
                let value = args.next().ok_or("--bind requires an address")?;
                config.bind = value.parse::<SocketAddr>()?;
            }
            "--public-url" => {
                config.public_base_url = Some(args.next().ok_or("--public-url requires a URL")?);
            }
            "--help" | "-h" => {
                println!("usage: pulselink-relay [--bind ADDR] [--public-url URL]");
                return Ok(());
            }
            other => return Err(format!("unknown argument: {other}").into()),
        }
    }

    let relay = LocalRelay::start(config).await?;
    println!("controller: {}", relay.controller_url());
    println!("press Ctrl-C to stop");
    tokio::signal::ctrl_c().await?;
    relay.shutdown().await?;
    Ok(())
}
