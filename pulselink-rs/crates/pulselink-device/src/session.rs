//! Shared local-session state, events, and notification pumps.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use pulselink_core::{
    BatteryState, ButtonMask, Channel, ConnectionState, DeviceState, Intensity, IntensitySource,
    Observed, Pressure, Trigger,
};
use pulselink_transport::{GattLink, NotificationStream};
use tokio::sync::{broadcast, watch};
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::error::DeviceError;

/// An event emitted by a local device session.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum LocalEvent {
    /// Connection state changed.
    ConnectionChanged(ConnectionState),
    /// Battery state changed.
    BatteryChanged(BatteryState),
    /// A channel intensity changed.
    IntensityChanged {
        /// Affected channel.
        channel: Channel,
        /// Observed intensity.
        intensity: Observed<Intensity>,
        /// Source of the change.
        source: IntensitySource,
    },
    /// A pressure reading changed.
    PressureChanged(Observed<Pressure>),
    /// A physical button state changed.
    ButtonChanged(ButtonMask),
    /// A trigger event fired.
    Trigger(Trigger),
    /// Raw sensor data was received.
    SensorData,
    /// A non-fatal error occurred.
    Error(String),
    /// The session closed.
    Closed,
}

/// State shared by the lightweight local-device sessions.
pub(crate) struct SessionShared {
    pub(crate) link: Arc<dyn GattLink>,
    state_tx: watch::Sender<DeviceState>,
    events: broadcast::Sender<LocalEvent>,
    pumps: Mutex<Vec<JoinHandle<()>>>,
    closed: AtomicBool,
}

impl SessionShared {
    pub(crate) fn new(link: Box<dyn GattLink>) -> Arc<Self> {
        let link: Arc<dyn GattLink> = Arc::from(link);
        let (state_tx, _) = watch::channel(DeviceState::new(ConnectionState::Initializing));
        let (events, _) = broadcast::channel(256);
        Arc::new(Self {
            link,
            state_tx,
            events,
            pumps: Mutex::new(Vec::new()),
            closed: AtomicBool::new(false),
        })
    }

    pub(crate) fn state(&self) -> DeviceState {
        self.state_tx.borrow().clone()
    }

    pub(crate) fn subscribe(&self) -> broadcast::Receiver<LocalEvent> {
        self.events.subscribe()
    }

    pub(crate) fn emit(&self, event: LocalEvent) {
        let _ = self.events.send(event);
    }

    pub(crate) fn set_connection(&self, connection: ConnectionState) {
        self.state_tx.send_modify(|state| {
            state.connection = connection;
            state.touch();
        });
        self.emit(LocalEvent::ConnectionChanged(connection));
    }

    pub(crate) fn set_battery(&self, battery: BatteryState) {
        self.state_tx.send_modify(|state| {
            state.battery = Observed::Known(battery);
            state.touch();
        });
        self.emit(LocalEvent::BatteryChanged(battery));
    }

    pub(crate) fn set_intensity(
        &self,
        channel: Channel,
        intensity: Intensity,
        source: IntensitySource,
    ) {
        self.state_tx.send_modify(|state| {
            state.channels.get_mut(channel).intensity = Observed::Known(intensity);
            state.touch();
        });
        self.emit(LocalEvent::IntensityChanged {
            channel,
            intensity: Observed::Known(intensity),
            source,
        });
    }

    pub(crate) fn ensure_open(&self) -> Result<(), DeviceError> {
        if self.closed.load(Ordering::SeqCst) {
            Err(DeviceError::Closed)
        } else {
            Ok(())
        }
    }

    pub(crate) fn spawn_pump<F>(self: &Arc<Self>, characteristic: Uuid, mut on_data: F)
    where
        F: FnMut(&[u8]) + Send + 'static,
    {
        let link = Arc::clone(&self.link);
        let shared = Arc::clone(self);
        let handle = tokio::spawn(async move {
            let mut stream: NotificationStream = match link.subscribe(characteristic).await {
                Ok(stream) => stream,
                Err(error) => {
                    shared.emit(LocalEvent::Error(error.to_string()));
                    return;
                }
            };
            while let Some(item) = futures_util::StreamExt::next(&mut stream).await {
                match item {
                    Ok(bytes) => on_data(&bytes),
                    Err(error) => {
                        shared.emit(LocalEvent::Error(error.to_string()));
                        break;
                    }
                }
            }
        });
        self.pumps
            .lock()
            .expect("session pump mutex poisoned")
            .push(handle);
    }

    pub(crate) async fn shutdown(&self) -> Result<(), DeviceError> {
        if self.closed.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        self.state_tx.send_modify(|state| {
            state.connection = ConnectionState::Closing;
            state.touch();
        });
        for handle in self
            .pumps
            .lock()
            .expect("session pump mutex poisoned")
            .drain(..)
        {
            handle.abort();
        }
        let result = self.link.disconnect().await;
        self.state_tx.send_modify(|state| {
            state.connection = ConnectionState::Disconnected;
            state.touch();
        });
        self.emit(LocalEvent::Closed);
        result.map_err(DeviceError::from)
    }
}

/// A connected local device session.
#[derive(Clone)]
#[non_exhaustive]
pub enum LocalSession {
    /// Coyote V2 BLE.
    CoyoteV2(crate::coyote_v2::CoyoteV2Session),
    /// Coyote V3 BLE, backed by `pulselink-runtime`.
    CoyoteV3(pulselink_runtime::CoyoteV3Session),
    /// Opossum BLE.
    Opossum(crate::opossum::OpossumSession),
    /// Civet BLE.
    Civet(crate::civet::CivetSession),
    /// `PawPrints` BLE.
    PawPrints(crate::paw_prints::PawPrintsSession),
}

impl LocalSession {
    /// Returns the device kind.
    #[must_use]
    pub fn kind(&self) -> pulselink_core::DeviceKind {
        match self {
            Self::CoyoteV2(session) => session.kind(),
            Self::CoyoteV3(session) => session.device_info().kind.clone(),
            Self::Opossum(session) => session.kind(),
            Self::Civet(session) => session.kind(),
            Self::PawPrints(session) => session.kind(),
        }
    }

    /// Returns the latest common state.
    #[must_use]
    pub fn state(&self) -> DeviceState {
        match self {
            Self::CoyoteV2(session) => session.state(),
            Self::CoyoteV3(session) => session.state().common.clone(),
            Self::Opossum(session) => session.state(),
            Self::Civet(session) => session.state(),
            Self::PawPrints(session) => session.state(),
        }
    }

    /// Shuts down the session.
    pub async fn shutdown(self) -> Result<(), DeviceError> {
        match self {
            Self::CoyoteV2(session) => session.shutdown().await,
            Self::CoyoteV3(session) => session.shutdown().await.map_err(DeviceError::from),
            Self::Opossum(session) => session.shutdown().await,
            Self::Civet(session) => session.shutdown().await,
            Self::PawPrints(session) => session.shutdown().await,
        }
    }
}
