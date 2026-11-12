//! Local session tests using the transport mock.

use std::sync::Arc;
use std::time::Duration;

use pulselink_core::{Channel, DeviceKind};
use pulselink_device::{DeviceClient, LocalSession};
use pulselink_protocol::civet::{CHAR_NOTIFY as CIVET_NOTIFY, D0Message as CivetD0};
use pulselink_protocol::coyote::v2::{CHAR_PWM_AB2, CoyoteV2Intensity, PwmAb2, SERVICE_PWM};
use pulselink_protocol::coyote::v3::gatt::SERVICE_CONTROL;
use pulselink_protocol::opossum::{CHAR_NOTIFY as OPOSSUM_NOTIFY, CHAR_WRITE as OPOSSUM_WRITE};
use pulselink_protocol::paw_prints::{
    CHAR_NOTIFY as PAW_NOTIFY, CHAR_WRITE as PAW_WRITE, Message51,
};
use pulselink_transport::mock::{MockBackend, MockLink};
use pulselink_transport::{DiscoveredDevice, TransportDeviceId};
use tokio::time::sleep;

fn discovered(name: &str, service: uuid::Uuid) -> DiscoveredDevice {
    DiscoveredDevice {
        id: TransportDeviceId::new(name),
        name: Some(name.to_owned()),
        rssi: Some(-40),
        service_uuids: vec![service],
        manufacturer_data: Vec::new(),
    }
}

fn client_with(link: &MockLink, devices: Vec<DiscoveredDevice>) -> DeviceClient {
    let backend = MockBackend::new(devices);
    backend.push_link(link.clone());
    DeviceClient::new(Arc::new(backend))
}

#[tokio::test]
async fn coyote_v2_writes_intensity_and_reads_notifications() {
    let device = discovered("D-LAB ESTIM01", SERVICE_PWM);
    let link = MockLink::new(device.id.clone());
    let client = client_with(&link, vec![device.clone()]);
    let session = client.connect(&device).await.expect("connect");
    let LocalSession::CoyoteV2(session) = session else {
        panic!("expected Coyote V2");
    };
    assert_eq!(session.kind(), DeviceKind::CoyoteV2);

    let intensity = CoyoteV2Intensity::from_display(10).expect("intensity");
    session
        .set_intensity(Channel::A, intensity)
        .await
        .expect("set intensity");
    let write = link
        .writes()
        .into_iter()
        .find(|write| write.characteristic == CHAR_PWM_AB2)
        .expect("PWM_AB2 write");
    assert_eq!(write.data, [0, 0x30, 0x02]);

    sleep(Duration::from_millis(20)).await;
    link.inject_notification(
        CHAR_PWM_AB2,
        &PwmAb2 {
            a: CoyoteV2Intensity::new(14).expect("a"),
            b: CoyoteV2Intensity::new(7).expect("b"),
        }
        .encode(),
    );
    sleep(Duration::from_millis(20)).await;
    let state = session.state();
    assert_eq!(
        state.channel(Channel::A).intensity.value().map(|v| v.get()),
        Some(2)
    );
    assert_eq!(
        state.channel(Channel::B).intensity.value().map(|v| v.get()),
        Some(1)
    );
}

#[tokio::test]
async fn opossum_writes_b3_and_b0() {
    let device = discovered("47L127000", SERVICE_CONTROL);
    let link = MockLink::new(device.id.clone());
    link.set_mtu(23);
    let client = client_with(&link, vec![device.clone()]);
    let session = client.connect(&device).await.expect("connect");
    let LocalSession::Opossum(session) = session else {
        panic!("expected Opossum");
    };

    session
        .set_pulse(Channel::A, [10, 20, 30, 40])
        .await
        .expect("pulse");
    let write = link
        .writes()
        .into_iter()
        .find(|write| write.data.first() == Some(&0xB0))
        .expect("B0");
    assert_eq!(&write.data[8..12], &[10, 20, 30, 40]);

    let intensity = pulselink_core::Intensity::new(80).expect("intensity");
    session
        .set_intensity(Channel::A, intensity)
        .await
        .expect("intensity");
    let writes = link.writes();
    assert!(
        writes
            .iter()
            .any(|write| write.characteristic == OPOSSUM_WRITE && write.data[0] == 0xB3)
    );
    assert!(!writes.iter().any(|write| write.data.first() == Some(&0xB2)));

    link.set_mtu(185);
    session
        .set_intensity(
            Channel::B,
            pulselink_core::Intensity::new(20).expect("intensity"),
        )
        .await
        .expect("intensity with MTU");
    assert!(
        link.writes()
            .iter()
            .any(|write| write.data.first() == Some(&0xB2) && write.data.len() == 24)
    );

    link.inject_notification(OPOSSUM_NOTIFY, &[0xB3, 80, 0]);
    sleep(Duration::from_millis(20)).await;
    assert_eq!(
        session
            .state()
            .channel(Channel::A)
            .intensity
            .value()
            .map(|v| v.get()),
        Some(80)
    );
}

#[tokio::test]
async fn civet_pressure_notification_updates_state() {
    let device = discovered("47L124000", SERVICE_CONTROL);
    let link = MockLink::new(device.id.clone());
    let client = client_with(&link, vec![device.clone()]);
    let session = client.connect(&device).await.expect("connect");
    let LocalSession::Civet(session) = session else {
        panic!("expected Civet");
    };
    sleep(Duration::from_millis(20)).await;
    let mut bytes = [0u8; 17];
    bytes[0] = 0xD0;
    bytes[8..10].copy_from_slice(&1234i16.to_le_bytes());
    link.inject_notification(CIVET_NOTIFY, &bytes);
    sleep(Duration::from_millis(20)).await;
    assert_eq!(
        session.pressure().value().map(|p| p.as_centi_kpa()),
        Some(1234)
    );
    assert_eq!(
        CivetD0::decode(&bytes)
            .expect("decode")
            .pressure
            .as_centi_kpa(),
        1234
    );
}

#[tokio::test]
async fn paw_prints_sends_no_trigger_and_decodes_battery() {
    let device = discovered("47L120300", SERVICE_CONTROL);
    let link = MockLink::new(device.id.clone());
    let client = client_with(&link, vec![device.clone()]);
    let session = client.connect(&device).await.expect("connect");
    let LocalSession::PawPrints(session) = session else {
        panic!("expected PawPrints");
    };
    let mut events = session.subscribe();

    let write = link
        .writes()
        .into_iter()
        .find(|write| write.characteristic == PAW_WRITE)
        .expect("no-trigger write");
    assert_eq!(&write.data[..3], &[0x50, 1, 0x00]);

    sleep(Duration::from_millis(20)).await;
    let info = Message51 {
        color: 1,
        device_type: 3,
        battery: 77,
    };
    link.inject_notification(
        PAW_NOTIFY,
        &[0x51, info.color, info.device_type, info.battery],
    );
    sleep(Duration::from_millis(20)).await;
    let snapshot = session.snapshot();
    assert_eq!(snapshot.battery.value().map(|b| b.get()), Some(77));
    assert!(events.try_recv().is_ok());
}
