//! Optional real-hardware smoke test.
//!
//! Run with `PULSELINK_HW=1` and a Coyote V3 nearby:
//! `cargo test -p pulselink-integration-tests --test hardware -- --ignored`

use std::sync::Arc;
use std::time::Duration;

use pulselink_ble::BtleplugCentral;
use pulselink_core::ConnectionState;
use pulselink_protocol::coyote::v3::gatt::{DEVICE_NAME, SERVICE_CONTROL};
use pulselink_runtime::{Client, CoyoteV3Config};
use pulselink_transport::ScanFilter;

#[tokio::test]
#[ignore = "requires PULSELINK_HW=1 and a real Coyote V3"]
async fn hardware_scan_connect_read_battery() {
    if std::env::var("PULSELINK_HW").as_deref() != Ok("1") {
        return;
    }

    let backend = Arc::new(BtleplugCentral::new().await.expect("adapter"));
    let client = Client::new(backend);
    let devices = client
        .scan(
            ScanFilter {
                name_prefixes: vec![DEVICE_NAME.to_owned()],
                service_uuids: vec![SERVICE_CONTROL],
            },
            Duration::from_secs(5),
        )
        .await
        .expect("scan");
    let device = devices.first().expect("no Coyote V3 found");
    let session = client
        .connect_coyote_v3(device, CoyoteV3Config::default())
        .await
        .expect("connect");
    assert_eq!(session.state().common.connection, ConnectionState::Ready);
    assert!(session.state().common.battery.is_known());
    session.shutdown().await.expect("shutdown");
    client.shutdown().await;
}

#[tokio::test]
#[ignore = "requires PULSELINK_HW=1 and a real Coyote V3"]
async fn hardware_raw_gatt_battery_notify_bf_zero_b0() {
    use pulselink_core::{ChannelMap, Intensity};
    use pulselink_protocol::coyote::v3::gatt::{CHAR_BATTERY, CHAR_NOTIFY, CHAR_WRITE};
    use pulselink_protocol::coyote::v3::{
        B0Command, BfCommand, ChannelWaveform, IntensityOp, Sequence, StereoIntensityOp,
        StereoWaveform,
    };
    use pulselink_transport::{Central, GattLink, WriteMode};

    if std::env::var("PULSELINK_HW").as_deref() != Ok("1") {
        return;
    }

    let central = BtleplugCentral::new().await.expect("adapter");
    let devices = central
        .scan(
            ScanFilter {
                name_prefixes: vec![DEVICE_NAME.to_owned()],
                service_uuids: vec![SERVICE_CONTROL],
            },
            Duration::from_secs(5),
        )
        .await
        .expect("scan");
    let device = devices.first().expect("no Coyote V3 found");
    let link = central.connect_link(device).await.expect("connect");

    let battery = link.read(CHAR_BATTERY).await.expect("battery read");
    assert_ne!(battery.len(), 0);
    let _notifications = link.subscribe(CHAR_NOTIFY).await.expect("subscribe");

    let bf = BfCommand {
        soft_limit: ChannelMap::new(Intensity::MIN, Intensity::MIN),
        frequency_balance: ChannelMap::new(128, 128),
        intensity_balance: ChannelMap::new(128, 128),
    };
    link.write(CHAR_WRITE, &bf.encode(), WriteMode::Auto)
        .await
        .expect("BF write");

    let b0 = B0Command {
        sequence: Sequence::new(0).expect("sequence"),
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
        .await
        .expect("zero B0 write");
    link.disconnect().await.expect("disconnect");
}

#[tokio::test]
#[ignore = "requires PULSELINK_HW=1 and real Coyote V2/PawPrints/Civet/Opossum devices"]
async fn hardware_new_device_sessions_initialize() {
    use pulselink_device::{
        CIVET, COYOTE_V2, DeviceClient, DeviceProfile, LocalSession, OPOSSUM, PAW_PRINTS,
    };

    if std::env::var("PULSELINK_HW").as_deref() != Ok("1") {
        return;
    }

    let backend = Arc::new(BtleplugCentral::new().await.expect("adapter"));
    let client = DeviceClient::new(backend);
    for profile in [&COYOTE_V2, &PAW_PRINTS, &CIVET, &OPOSSUM] {
        let label = profile.kind.as_str();
        let devices = client
            .scan_profile(profile as &DeviceProfile, Duration::from_secs(3))
            .await
            .expect("scan");
        let Some(device) = devices.first() else {
            eprintln!("{label}: not found; skipped");
            continue;
        };
        let session = client.connect(device).await.expect("connect");
        assert_eq!(
            session.state().connection,
            ConnectionState::Ready,
            "{label}"
        );
        match session {
            LocalSession::CoyoteV2(session) => session.shutdown().await.expect("shutdown"),
            LocalSession::PawPrints(session) => session.shutdown().await.expect("shutdown"),
            LocalSession::Civet(session) => session.shutdown().await.expect("shutdown"),
            LocalSession::Opossum(session) => session.shutdown().await.expect("shutdown"),
            other => other.shutdown().await.expect("shutdown"),
        }
    }
}
