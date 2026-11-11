//! Hardware probe for a Coyote V3 device.
//!
//! Set `PULSELINK_HW=1` before running. By default this example refuses to
//! connect, so a normal `cargo test --all-targets` is safe on machines without
//! a device.

use std::time::Duration;

use futures_util::StreamExt;
use pulselink_ble::BtleplugCentral;
use pulselink_core::{ChannelMap, Intensity};
use pulselink_protocol::coyote::v3::gatt::{
    CHAR_BATTERY, CHAR_NOTIFY, CHAR_WRITE, SERVICE_CONTROL,
};
use pulselink_protocol::coyote::v3::{
    B0Command, BfCommand, ChannelWaveform, IntensityOp, Sequence, StereoIntensityOp, StereoWaveform,
};
use pulselink_transport::{Central, GattLink, ScanFilter, WriteMode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("PULSELINK_HW").as_deref() != Ok("1") {
        println!("set PULSELINK_HW=1 to run the Coyote V3 hardware probe");
        return Ok(());
    }

    let central = BtleplugCentral::new().await?;
    println!("adapter: {}", central.adapter_info().await?);

    let devices = central
        .scan(
            ScanFilter {
                name_prefixes: vec!["47L".to_owned()],
                service_uuids: vec![SERVICE_CONTROL],
            },
            Duration::from_secs(5),
        )
        .await?;
    let device = devices
        .iter()
        .find(|device| device.name.as_deref() == Some("47L121000"))
        .or_else(|| devices.first())
        .ok_or("no Coyote V3 device found")?;
    println!("device: {:?} {:?}", device.id, device.name);

    let link = central.connect_link(device).await?;
    println!("connected to {}", link.device_id());

    if let Some(info) = link.characteristic(CHAR_WRITE) {
        println!(
            "write characteristic: read={} write={} write_without_response={} \
             notify={} indicate={}",
            info.readable,
            info.writable,
            info.writable_without_response,
            info.notifiable,
            info.indicatable
        );
    }

    let battery = link.read(CHAR_BATTERY).await?;
    println!("battery: {battery:?}");

    let mut notifications = link.subscribe(CHAR_NOTIFY).await?;
    println!("subscribed to notifications");

    let bf = BfCommand {
        soft_limit: ChannelMap::new(Intensity::MIN, Intensity::MIN),
        frequency_balance: ChannelMap::new(128, 128),
        intensity_balance: ChannelMap::new(128, 128),
    };
    link.write(CHAR_WRITE, &bf.encode(), WriteMode::Auto)
        .await?;
    println!("BF written (soft limit 0)");

    let b0 = B0Command {
        sequence: Sequence::new(0)?,
        intensity: StereoIntensityOp {
            a: IntensityOp::Set(Intensity::MIN),
            b: IntensityOp::Set(Intensity::MIN),
        },
        waveform: StereoWaveform {
            a: ChannelWaveform::Silent,
            b: ChannelWaveform::Silent,
        },
    };
    link.write(CHAR_WRITE, &b0.encode(), WriteMode::Auto)
        .await?;
    println!("zero-intensity B0 written");

    if let Ok(Some(notification)) =
        tokio::time::timeout(Duration::from_secs(2), notifications.next()).await
    {
        match notification {
            Ok(value) => println!("notification: {value:02X?}"),
            Err(error) => println!("notification error: {error}"),
        }
    }

    link.disconnect().await?;
    println!("disconnected");
    Ok(())
}
