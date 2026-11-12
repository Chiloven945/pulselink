//! UI state and worker updates.

use std::time::Instant;

use pulselink::runtime::{CoyoteV3Event, CoyoteV3State};
use pulselink::transport::DiscoveredDevice;
use pulselink::ws::RemoteDeviceSnapshot;
use pulselink::{ButtonMask, DeviceKind, DeviceState, LocalEvent};

/// Log severity.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    /// Informational message.
    Info,
    /// Warning.
    Warn,
    /// Error.
    Error,
}

/// One line in the lab log.
#[derive(Clone, Debug)]
pub struct LogEntry {
    /// Time the entry was created.
    pub at: Instant,
    /// Severity.
    pub level: LogLevel,
    /// Message.
    pub message: String,
}

/// An update sent from the worker to the terminal UI.
#[derive(Clone, Debug)]
pub enum Update {
    /// New scan results.
    Devices(Vec<DiscoveredDevice>),
    /// Session connected.
    Connected {
        /// Device kind.
        kind: DeviceKind,
        /// Initial common state.
        state: DeviceState,
    },
    /// Common state changed.
    State(DeviceState),
    /// Coyote V3 state changed.
    V3State(CoyoteV3State),
    /// Local session event.
    LocalEvent(LocalEvent),
    /// Coyote V3 event.
    V3Event(CoyoteV3Event),
    /// V4 target and device cache.
    WsState {
        /// Controller target id.
        target: Option<String>,
        /// Typed snapshots from the app cache.
        devices: Vec<RemoteDeviceSnapshot>,
    },
    /// Log line.
    Log {
        /// Severity.
        level: LogLevel,
        /// Message.
        message: String,
    },
    /// Fatal or command error.
    Error(String),
    /// Session disconnected.
    Disconnected,
}

/// UI state derived from worker updates.
pub struct LabState {
    /// Scan results.
    pub devices: Vec<DiscoveredDevice>,
    /// Selected device index.
    pub selected: usize,
    /// Connected device kind.
    pub connected: Option<DeviceKind>,
    /// Latest common state.
    pub common: Option<DeviceState>,
    /// Latest V3 state.
    pub v3: Option<CoyoteV3State>,
    /// Pressure history in kPa.
    pub pressure: Vec<f32>,
    /// Latest button mask.
    pub buttons: ButtonMask,
    /// V4 target id.
    pub ws_target: Option<String>,
    /// V4 device snapshots.
    pub ws_devices: Vec<RemoteDeviceSnapshot>,
    /// Log entries.
    pub log: Vec<LogEntry>,
    /// Whether real output is enabled.
    pub output_enabled: bool,
    /// Maximum allowed intensity.
    pub max_intensity: u8,
    /// Last error.
    pub last_error: Option<String>,
}

impl Default for LabState {
    fn default() -> Self {
        Self {
            devices: Vec::new(),
            selected: 0,
            connected: None,
            common: None,
            v3: None,
            pressure: Vec::new(),
            buttons: ButtonMask::NONE,
            ws_target: None,
            ws_devices: Vec::new(),
            log: Vec::new(),
            output_enabled: false,
            max_intensity: 20,
            last_error: None,
        }
    }
}

impl LabState {
    /// Applies one worker update.
    pub fn apply(&mut self, update: Update) {
        match update {
            Update::Devices(devices) => {
                self.devices = devices;
                self.selected = self.selected.min(self.devices.len().saturating_sub(1));
            }
            Update::Connected { kind, state } => {
                self.log_info(format!("connected: {kind:?}"));
                self.connected = Some(kind);
                self.common = Some(state);
                self.v3 = None;
                self.pressure.clear();
            }
            Update::State(state) => self.common = Some(state),
            Update::V3State(state) => {
                self.common = Some(state.common.clone());
                self.v3 = Some(state);
            }
            Update::LocalEvent(event) => self.apply_local_event(event),
            Update::V3Event(event) => self.log_info(format!("V3 event: {event:?}")),
            Update::WsState { target, devices } => {
                self.ws_target = target;
                self.ws_devices = devices;
            }
            Update::Log { level, message } => self.push_log(level, message),
            Update::Error(message) => {
                self.last_error = Some(message.clone());
                self.push_log(LogLevel::Error, message);
            }
            Update::Disconnected => {
                self.connected = None;
                self.common = None;
                self.v3 = None;
                self.push_log(LogLevel::Info, "disconnected".to_owned());
            }
        }
    }

    fn apply_local_event(&mut self, event: LocalEvent) {
        match event {
            LocalEvent::PressureChanged(value) => {
                if let Some(pressure) = value.value() {
                    self.pressure.push(pressure.as_kpa());
                    if self.pressure.len() > 120 {
                        self.pressure.remove(0);
                    }
                }
            }
            LocalEvent::ButtonChanged(buttons) => self.buttons = buttons,
            LocalEvent::Trigger(trigger) => {
                self.log_info(format!("trigger {:?}", trigger.event_id));
            }
            LocalEvent::Error(message) => self.push_log(LogLevel::Error, message),
            LocalEvent::Closed => {
                self.connected = None;
            }
            other => self.log_info(format!("event: {other:?}")),
        }
    }

    /// Appends an informational log line.
    pub fn log_info(&mut self, message: String) {
        self.push_log(LogLevel::Info, message);
    }

    fn push_log(&mut self, level: LogLevel, message: String) {
        self.log.push(LogEntry {
            at: Instant::now(),
            level,
            message,
        });
        if self.log.len() > 500 {
            self.log.remove(0);
        }
    }

    /// Returns the selected device, when available.
    #[must_use]
    pub fn selected_device(&self) -> Option<&DiscoveredDevice> {
        self.devices.get(self.selected)
    }

    /// Returns the last pressure value.
    #[must_use]
    pub fn last_pressure(&self) -> Option<f32> {
        self.pressure.last().copied()
    }
}
