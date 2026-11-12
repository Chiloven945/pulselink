//! Civet V1 local pressure-sensor session.

use std::sync::{Arc, Mutex};

use pulselink_core::{BatteryState, DeviceKind, DeviceState, Observed, Pressure};
use pulselink_protocol::civet::{
    CHAR_BATTERY, CHAR_NOTIFY, CHAR_WRITE, Command50, Command66, D0Message,
};
use pulselink_protocol::coyote::v3::decode_battery;
use pulselink_transport::{GattLink, WriteMode};
use tokio::sync::broadcast;

use crate::error::DeviceError;
use crate::session::{LocalEvent, SessionShared};

/// A local Civet pressure session.
#[derive(Clone)]
pub struct CivetSession {
    shared: Arc<SessionShared>,
    pressure: Arc<Mutex<Observed<Pressure>>>,
}

impl CivetSession {
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
                    start: true,
                }
                .encode(),
                WriteMode::Auto,
            )
            .await?;

        let pressure = Arc::new(Mutex::new(Observed::Unknown));
        let pressure_events = Arc::clone(&shared);
        let pressure_state = Arc::clone(&pressure);
        shared.spawn_pump(CHAR_NOTIFY, move |bytes| match D0Message::decode(bytes) {
            Ok(message) => {
                *pressure_state
                    .lock()
                    .expect("Civet pressure mutex poisoned") = Observed::Known(message.pressure);
                pressure_events.emit(LocalEvent::PressureChanged(Observed::Known(
                    message.pressure,
                )));
            }
            Err(error) => pressure_events.emit(LocalEvent::Error(error.to_string())),
        });

        let battery_events = Arc::clone(&shared);
        shared.spawn_pump(CHAR_BATTERY, move |bytes| {
            if let Ok(level) = decode_battery(bytes) {
                battery_events.set_battery(BatteryState::new(level));
            }
        });

        shared.set_connection(pulselink_core::ConnectionState::Ready);
        Ok(Self { shared, pressure })
    }

    /// Returns the device kind.
    #[must_use]
    pub const fn kind(&self) -> DeviceKind {
        DeviceKind::Civet
    }

    /// Returns the latest common state.
    #[must_use]
    pub fn state(&self) -> DeviceState {
        self.shared.state()
    }

    /// Returns the last observed pressure.
    #[must_use]
    pub fn pressure(&self) -> Observed<Pressure> {
        *self.pressure.lock().expect("Civet pressure mutex poisoned")
    }

    /// Subscribes to session events.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<LocalEvent> {
        self.shared.subscribe()
    }

    /// Starts pressure reporting.
    pub async fn start_pressure(&self) -> Result<(), DeviceError> {
        self.send_command50(true).await
    }

    /// Stops pressure reporting.
    pub async fn stop_pressure(&self) -> Result<(), DeviceError> {
        self.send_command50(false).await
    }

    async fn send_command50(&self, start: bool) -> Result<(), DeviceError> {
        self.shared.ensure_open()?;
        self.shared
            .link
            .write(
                CHAR_WRITE,
                &Command50 { color: 1, start }.encode(),
                WriteMode::Auto,
            )
            .await?;
        Ok(())
    }

    /// Resets the pressure reading.
    pub async fn reset_pressure(&self) -> Result<(), DeviceError> {
        self.send_command66(Command66 {
            orientation: None,
            reset: true,
        })
        .await
    }

    /// Sets the display orientation.
    pub async fn set_orientation(&self, orientation: u8) -> Result<(), DeviceError> {
        self.send_command66(Command66 {
            orientation: Some(orientation),
            reset: false,
        })
        .await
    }

    async fn send_command66(&self, command: Command66) -> Result<(), DeviceError> {
        self.shared.ensure_open()?;
        self.shared
            .link
            .write(CHAR_WRITE, &command.encode()?, WriteMode::Auto)
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

    /// Stops reporting and disconnects.
    pub async fn shutdown(&self) -> Result<(), DeviceError> {
        let _ = self.send_command50(false).await;
        self.shared.shutdown().await
    }
}
