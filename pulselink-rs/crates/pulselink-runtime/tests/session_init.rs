//! Session initialization and safety tests.

mod common;

use std::sync::Arc;
use std::time::Duration;

use pulselink_core::{Channel, ChannelMap, ConnectionState, Intensity};
use pulselink_protocol::coyote::v3::gatt::{CHAR_BATTERY, CHAR_WRITE};
use pulselink_runtime::{Client, ClientConfig, SafetyPolicy};
use pulselink_transport::TransportDeviceId;
use pulselink_transport::mock::{MockBackend, MockLink};

use common::{config, device};

#[tokio::test(start_paused = true)]
async fn initializes_subscribes_battery_and_writes_bf() {
    let link = MockLink::new(TransportDeviceId::new("coyote-1"));
    link.set_read_data(CHAR_BATTERY, vec![88]);
    let backend = Arc::new(MockBackend::new(vec![device()]));
    backend.push_link(link.clone());

    let client = Client::with_config(
        backend,
        ClientConfig {
            connect_timeout: Duration::from_secs(1),
            ..ClientConfig::default()
        },
    );
    let session = client
        .connect_coyote_v3(&device(), config())
        .await
        .expect("connect");

    let state = session.state();
    assert_eq!(state.common.connection, ConnectionState::Ready);
    assert_eq!(
        state
            .common
            .battery
            .value()
            .map(|battery| battery.level.get()),
        Some(88)
    );

    let writes = link.writes();
    assert!(
        writes
            .iter()
            .any(|write| write.characteristic == CHAR_WRITE && write.data.first() == Some(&0xBF))
    );
}

#[tokio::test(start_paused = true)]
async fn safety_policy_rejects_above_limit() {
    let link = MockLink::new(TransportDeviceId::new("coyote-1"));
    link.set_read_data(CHAR_BATTERY, vec![80]);
    let backend = Arc::new(MockBackend::new(vec![device()]));
    backend.push_link(link.clone());
    let client = Client::with_config(backend, ClientConfig::default());
    let mut session_config = config();
    session_config.safety = SafetyPolicy {
        max_intensity: ChannelMap::new(Intensity::new(10).expect("valid"), Intensity::MAX),
        ..SafetyPolicy::default()
    };
    let session = client
        .connect_coyote_v3(&device(), session_config)
        .await
        .expect("connect");

    let result = session
        .set_intensity(Channel::A, Intensity::new(11).expect("valid"))
        .await;
    assert!(matches!(
        result,
        Err(pulselink_runtime::RuntimeError::SafetyLimit(_))
    ));
}
