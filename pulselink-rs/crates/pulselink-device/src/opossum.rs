//! Opossum V1 local BLE session.

use std::sync::{Arc, Mutex};

use pulselink_core::{
    BatteryState, Channel, ChannelMap, DeviceKind, DeviceState, Intensity, IntensitySource,
};
use pulselink_protocol::coyote::v3::decode_battery;
use pulselink_protocol::opossum::{
    B0, B2, B3, CHAR_BATTERY, CHAR_NOTIFY, CHAR_WRITE, Command50, Message as OpossumMessage,
};
use pulselink_transport::{GattLink, WriteMode};
use tokio::sync::broadcast;

use crate::error::DeviceError;
use crate::session::{LocalEvent, SessionShared};

const DEFAULT_PULSE: [u8; 4] = [0, 0, 0, 0];

/// A local Opossum V1 vibration session.
#[derive(Clone)]
pub struct OpossumSession {
    shared: Arc<SessionShared>,
    intensities: Arc<Mutex<ChannelMap<u8>>>,
    pulses: Arc<Mutex<ChannelMap<[u8; 4]>>>,
}

impl OpossumSession {
    pub(crate) async fn start(link: Box<dyn GattLink>) -> Result<Self, DeviceError> {
        let shared = SessionShared::new(link);
        if let Ok(bytes) = shared.link.read(CHAR_BATTERY).await
            && let Ok(level) = decode_battery(&bytes)
        {
            shared.set_battery(BatteryState::new(level));
        }

        shared
            .link
            .write(
                CHAR_WRITE,
                &Command50 {
                    color: 1,
                    button_report: true,
                }
                .encode(),
                WriteMode::Auto,
            )
            .await?;

        let intensities = Arc::new(Mutex::new(ChannelMap::new(0, 0)));
        let shared_events = Arc::clone(&shared);
        let observed_intensities = Arc::clone(&intensities);
        shared.spawn_pump(CHAR_NOTIFY, move |bytes| {
            match OpossumMessage::decode(bytes) {
                Ok(OpossumMessage::B3(message)) => {
                    {
                        let mut values = observed_intensities
                            .lock()
                            .expect("Opossum intensity mutex poisoned");
                        values.a = message.a;
                        values.b = message.b;
                    }
                    if let Ok(value) = Intensity::new(message.a) {
                        shared_events.set_intensity(Channel::A, value, IntensitySource::Device);
                    }
                    if let Ok(value) = Intensity::new(message.b) {
                        shared_events.set_intensity(Channel::B, value, IntensitySource::Device);
                    }
                }
                Ok(OpossumMessage::D0(message)) => {
                    shared_events.emit(LocalEvent::ButtonChanged(message.buttons));
                }
                Err(error) => shared_events.emit(LocalEvent::Error(error.to_string())),
            }
        });

        let shared_battery = Arc::clone(&shared);
        shared.spawn_pump(CHAR_BATTERY, move |bytes| {
            if let Ok(level) = decode_battery(bytes) {
                shared_battery.set_battery(BatteryState::new(level));
            }
        });

        shared.set_connection(pulselink_core::ConnectionState::Ready);
        Ok(Self {
            shared,
            intensities,
            pulses: Arc::new(Mutex::new(ChannelMap::new(DEFAULT_PULSE, DEFAULT_PULSE))),
        })
    }

    /// Returns the device kind.
    #[must_use]
    pub const fn kind(&self) -> DeviceKind {
        DeviceKind::Opossum
    }

    /// Returns the latest common state.
    #[must_use]
    pub fn state(&self) -> DeviceState {
        self.shared.state()
    }

    /// Subscribes to session events.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<LocalEvent> {
        self.shared.subscribe()
    }

    /// Sets one channel intensity and syncs the display when MTU allows it.
    pub async fn set_intensity(
        &self,
        channel: Channel,
        intensity: Intensity,
    ) -> Result<(), DeviceError> {
        self.shared.ensure_open()?;
        let value = intensity.get().min(200);
        let both = {
            let mut intensities = self
                .intensities
                .lock()
                .expect("Opossum intensity mutex poisoned");
            *intensities.get_mut(channel) = value;
            *intensities
        };
        let command = B3 {
            a: Some(both.a),
            b: Some(both.b),
        }
        .encode()?;
        self.shared
            .link
            .write(CHAR_WRITE, &command, WriteMode::Auto)
            .await?;
        if self.shared.link.mtu().is_some_and(|mtu| mtu >= 24) {
            let display = B2 {
                a: both.a,
                b: both.b,
            }
            .encode()?;
            let _ = self
                .shared
                .link
                .write(CHAR_WRITE, &display, WriteMode::Auto)
                .await;
        }
        self.shared
            .set_intensity(channel, intensity, IntensitySource::Sdk);
        Ok(())
    }

    /// Sets one channel's 100 ms vibration frame while preserving the other.
    pub async fn set_pulse(&self, channel: Channel, frames: [u8; 4]) -> Result<(), DeviceError> {
        self.shared.ensure_open()?;
        let both = {
            let mut pulses = self.pulses.lock().expect("Opossum pulse mutex poisoned");
            *pulses.get_mut(channel) = frames;
            *pulses
        };
        let command = B0 {
            a: both.a,
            b: both.b,
        }
        .encode()?;
        self.shared
            .link
            .write(CHAR_WRITE, &command, WriteMode::Auto)
            .await?;
        Ok(())
    }

    /// Reads and updates the battery state.
    pub async fn refresh_battery(&self) -> Result<(), DeviceError> {
        self.shared.ensure_open()?;
        let bytes = self.shared.link.read(CHAR_BATTERY).await?;
        let level = decode_battery(&bytes)?;
        self.shared.set_battery(BatteryState::new(level));
        Ok(())
    }

    /// Disconnects and attempts a zero-output stop.
    pub async fn shutdown(&self) -> Result<(), DeviceError> {
        if self.shared.ensure_open().is_ok() {
            let zero = B0 {
                a: DEFAULT_PULSE,
                b: DEFAULT_PULSE,
            }
            .encode()?;
            let _ = self
                .shared
                .link
                .write(CHAR_WRITE, &zero, WriteMode::Auto)
                .await;
            let _ = self
                .shared
                .link
                .write(
                    CHAR_WRITE,
                    &B3 {
                        a: Some(0),
                        b: Some(0),
                    }
                    .encode()?,
                    WriteMode::Auto,
                )
                .await;
        }
        self.shared.shutdown().await
    }
}
