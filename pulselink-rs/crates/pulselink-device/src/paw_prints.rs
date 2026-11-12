//! `PawPrints` V1.1 local button/sensor session.

use std::sync::{Arc, Mutex};

use pulselink_core::{
    BatteryLevel, BatteryState, ButtonMask, DeviceKind, DeviceState, Observed, Trigger,
};
use pulselink_protocol::paw_prints::{
    CHAR_NOTIFY, CHAR_WRITE, COMMAND_AUTO_DETECT_ANGLES, COMMAND_RESET_PARAMETER, Command50,
    D0Message, F1Message, Message, Message5a, Message5b, Message5c,
};
use pulselink_transport::{GattLink, WriteMode};
use tokio::sync::broadcast;

use crate::error::DeviceError;
use crate::session::{LocalEvent, SessionShared};

/// Optional sensor data reported by `PawPrints`.
#[derive(Clone, Debug, Default)]
pub struct PawPrintsSnapshot {
    /// Battery level.
    pub battery: Observed<BatteryLevel>,
    /// Last raw sensor reading.
    pub sensor: Option<D0Message>,
    /// Last activation event.
    pub last_activation: Option<Message5a>,
    /// Last deactivation event.
    pub last_deactivation: Option<Message5b>,
    /// Last parameter event.
    pub last_parameter: Option<Message5c>,
    /// Last auto-detected angle thresholds.
    pub angles: Option<F1Message>,
}

/// A local `PawPrints` session.
#[derive(Clone)]
pub struct PawPrintsSession {
    shared: Arc<SessionShared>,
    snapshot: Arc<Mutex<PawPrintsSnapshot>>,
}

impl PawPrintsSession {
    pub(crate) async fn start(link: Box<dyn GattLink>) -> Result<Self, DeviceError> {
        let shared = SessionShared::new(link);
        shared
            .link
            .write(
                CHAR_WRITE,
                Command50::no_trigger(1).as_bytes(),
                WriteMode::Auto,
            )
            .await?;

        let snapshot = Arc::new(Mutex::new(PawPrintsSnapshot::default()));
        let events = Arc::clone(&shared);
        let state = Arc::clone(&snapshot);
        shared.spawn_pump(CHAR_NOTIFY, move |bytes| match Message::decode(bytes) {
            Ok(Message::Info(info)) => {
                if let Ok(level) = BatteryLevel::new(info.battery) {
                    state
                        .lock()
                        .expect("PawPrints state mutex poisoned")
                        .battery = Observed::Known(level);
                    events.set_battery(BatteryState::new(level));
                }
            }
            Ok(Message::Activated(message)) => {
                state
                    .lock()
                    .expect("PawPrints state mutex poisoned")
                    .last_activation = Some(message);
                events.emit(LocalEvent::Trigger(Trigger {
                    event_id: message.event_id,
                    parameter: message.parameter,
                }));
            }
            Ok(Message::Deactivated(message)) => {
                state
                    .lock()
                    .expect("PawPrints state mutex poisoned")
                    .last_deactivation = Some(message);
                events.emit(LocalEvent::ButtonChanged(ButtonMask::NONE));
            }
            Ok(Message::Parameter(message)) => {
                state
                    .lock()
                    .expect("PawPrints state mutex poisoned")
                    .last_parameter = Some(message);
                events.emit(LocalEvent::Trigger(Trigger {
                    event_id: message.event_id,
                    parameter: message.parameter,
                }));
            }
            Ok(Message::Sensor(sensor)) => {
                state.lock().expect("PawPrints state mutex poisoned").sensor = Some(sensor);
                events.emit(LocalEvent::ButtonChanged(ButtonMask::from_bits(u16::from(
                    sensor.button != 0,
                ))));
                events.emit(LocalEvent::SensorData);
            }
            Ok(Message::Angles(angles)) => {
                state.lock().expect("PawPrints state mutex poisoned").angles = Some(angles);
            }
            Err(error) => events.emit(LocalEvent::Error(error.to_string())),
        });

        shared.set_connection(pulselink_core::ConnectionState::Ready);
        Ok(Self { shared, snapshot })
    }

    /// Returns the device kind.
    #[must_use]
    pub const fn kind(&self) -> DeviceKind {
        DeviceKind::PawPrints
    }

    /// Returns the latest common state.
    #[must_use]
    pub fn state(&self) -> DeviceState {
        self.shared.state()
    }

    /// Returns a snapshot of last-seen sensor/trigger state.
    #[must_use]
    pub fn snapshot(&self) -> PawPrintsSnapshot {
        self.snapshot
            .lock()
            .expect("PawPrints state mutex poisoned")
            .clone()
    }

    /// Subscribes to session events.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<LocalEvent> {
        self.shared.subscribe()
    }

    /// Sends a full trigger configuration.
    pub async fn configure(&self, command: Command50) -> Result<(), DeviceError> {
        self.shared.ensure_open()?;
        self.shared
            .link
            .write(CHAR_WRITE, command.as_bytes(), WriteMode::Auto)
            .await?;
        Ok(())
    }

    /// Resets the current trigger parameter.
    pub async fn reset_parameter(&self) -> Result<(), DeviceError> {
        self.write_command(&COMMAND_RESET_PARAMETER).await
    }

    /// Starts automatic angle threshold detection.
    pub async fn auto_detect_angles(&self) -> Result<(), DeviceError> {
        self.write_command(&COMMAND_AUTO_DETECT_ANGLES).await
    }

    /// Sets a steady shoulder-light color.
    pub async fn set_shoulder_light(&self, color: u8) -> Result<(), DeviceError> {
        self.write_command(&[0x70, color]).await
    }

    /// Configures flashing shoulder lights.
    pub async fn flash_shoulder_light(
        &self,
        color_a: u8,
        color_b: u8,
        speed: u8,
    ) -> Result<(), DeviceError> {
        self.write_command(&[0x70, color_a, color_b, speed]).await
    }

    async fn write_command(&self, bytes: &[u8]) -> Result<(), DeviceError> {
        self.shared.ensure_open()?;
        self.shared
            .link
            .write(CHAR_WRITE, bytes, WriteMode::Auto)
            .await?;
        Ok(())
    }

    /// Disconnects and stops background pumps.
    pub async fn shutdown(&self) -> Result<(), DeviceError> {
        self.shared.shutdown().await
    }
}
