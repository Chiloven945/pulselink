//! Coyote V2 local BLE session.

use std::sync::{Arc, Mutex};

use pulselink_core::{
    BatteryState, Channel, ChannelMap, DeviceKind, DeviceState, Intensity, IntensitySource,
    Observed,
};
use pulselink_protocol::coyote::v2::{
    CHAR_BATTERY, CHAR_PWM_A34, CHAR_PWM_AB2, CHAR_PWM_B34, CoyoteV2Intensity, PwmAb2, PwmPulse,
};
use pulselink_protocol::coyote::v3::decode_battery;
use pulselink_transport::{GattLink, WriteMode};
use tokio::sync::broadcast;

use crate::error::DeviceError;
use crate::session::{LocalEvent, SessionShared};

/// A local Coyote V2 session.
#[derive(Clone)]
pub struct CoyoteV2Session {
    shared: Arc<SessionShared>,
    intensities: Arc<Mutex<ChannelMap<CoyoteV2Intensity>>>,
}

impl CoyoteV2Session {
    pub(crate) async fn start(link: Box<dyn GattLink>) -> Result<Self, DeviceError> {
        let shared = SessionShared::new(link);
        if let Ok(bytes) = shared.link.read(CHAR_BATTERY).await
            && let Ok(level) = decode_battery(&bytes)
        {
            shared.set_battery(BatteryState::new(level));
        }

        let intensities = Arc::new(Mutex::new(ChannelMap::new(
            CoyoteV2Intensity::MIN,
            CoyoteV2Intensity::MIN,
        )));
        let intensity_shared = Arc::clone(&shared);
        let observed_intensities = Arc::clone(&intensities);
        shared.spawn_pump(CHAR_PWM_AB2, move |bytes| {
            if let Ok(value) = PwmAb2::decode(bytes) {
                {
                    let mut values = observed_intensities
                        .lock()
                        .expect("Coyote V2 intensity mutex poisoned");
                    *values.get_mut(Channel::A) = value.a;
                    *values.get_mut(Channel::B) = value.b;
                }
                intensity_shared.set_intensity(
                    Channel::A,
                    display_intensity(value.a),
                    IntensitySource::Device,
                );
                intensity_shared.set_intensity(
                    Channel::B,
                    display_intensity(value.b),
                    IntensitySource::Device,
                );
            }
        });

        let battery_shared = Arc::clone(&shared);
        shared.spawn_pump(CHAR_BATTERY, move |bytes| {
            if let Ok(level) = decode_battery(bytes) {
                battery_shared.set_battery(BatteryState::new(level));
            }
        });

        shared.set_connection(pulselink_core::ConnectionState::Ready);
        Ok(Self {
            shared,
            intensities,
        })
    }

    /// Returns the device kind.
    #[must_use]
    pub const fn kind(&self) -> DeviceKind {
        DeviceKind::CoyoteV2
    }

    /// Returns the latest common device state.
    #[must_use]
    pub fn state(&self) -> DeviceState {
        self.shared.state()
    }

    /// Subscribes to session events.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<LocalEvent> {
        self.shared.subscribe()
    }

    /// Sets one channel's wire intensity while preserving the other channel.
    pub async fn set_intensity(
        &self,
        channel: Channel,
        intensity: CoyoteV2Intensity,
    ) -> Result<(), DeviceError> {
        self.shared.ensure_open()?;
        let value = {
            let mut intensities = self
                .intensities
                .lock()
                .expect("Coyote V2 intensity mutex poisoned");
            *intensities.get_mut(channel) = intensity;
            *intensities
        };
        let bytes = PwmAb2 {
            a: value.a,
            b: value.b,
        }
        .encode();
        self.shared
            .link
            .write(CHAR_PWM_AB2, &bytes, WriteMode::Auto)
            .await?;
        self.shared
            .set_intensity(channel, display_intensity(intensity), IntensitySource::Sdk);
        Ok(())
    }

    /// Sends one 100 ms pulse parameter set to a channel.
    pub async fn set_pulse(&self, channel: Channel, pulse: PwmPulse) -> Result<(), DeviceError> {
        self.shared.ensure_open()?;
        let characteristic = match channel {
            Channel::A => CHAR_PWM_B34,
            Channel::B => CHAR_PWM_A34,
        };
        self.shared
            .link
            .write(characteristic, &pulse.encode(), WriteMode::Auto)
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

    /// Returns the last intensity written/observed for a channel.
    #[must_use]
    pub fn intensity(&self, channel: Channel) -> Observed<CoyoteV2Intensity> {
        Observed::Known(
            *self
                .intensities
                .lock()
                .expect("Coyote V2 intensity mutex poisoned")
                .get(channel),
        )
    }

    /// Disconnects and stops background pumps.
    pub async fn shutdown(&self) -> Result<(), DeviceError> {
        self.shared.shutdown().await
    }
}

fn display_intensity(value: CoyoteV2Intensity) -> Intensity {
    Intensity::new(value.display()).unwrap_or(Intensity::MIN)
}
