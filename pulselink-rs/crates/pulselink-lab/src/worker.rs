//! Background device worker for the lab.

use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::JoinHandle;
use std::time::Duration;

use pulselink::device::profile_for;
use pulselink::pattern::{
    ChannelFrame, Pattern, PulseIntensity, PulseInterval, PulseSample, StereoFrame,
};
use pulselink::protocol::coyote::v2::{CoyoteV2Intensity, PwmPulse, SERVICE_PWM};
use pulselink::protocol::coyote::v3::gatt::SERVICE_CONTROL;
use pulselink::protocol::paw_prints::Command50;
use pulselink::runtime::PlayOptions;
use pulselink::transport::ScanFilter;
use pulselink::ws::{AppClient, ClearTarget, OperationOptions, RemoteDeviceSnapshot, Socket};
use pulselink::{Channel, DeviceKind, Intensity, LocalEvent, LocalSession, PulseLink};
use pulselink_relay::{LocalRelay, RelayConfig};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::task::JoinHandle as TokioJoinHandle;
use tokio::time::timeout;

use crate::command::Command;
use crate::model::{LogLevel, Update};

/// A handle to the background device worker.
pub struct LabWorker {
    commands: UnboundedSender<Command>,
    updates: Receiver<Update>,
    thread: JoinHandle<()>,
}

impl LabWorker {
    /// Spawns a worker using the first system Bluetooth adapter.
    #[must_use]
    pub fn spawn() -> Self {
        Self::spawn_inner(None)
    }

    /// Spawns a worker with an explicit facade instance.
    #[must_use]
    #[allow(dead_code)]
    pub fn spawn_with(link: PulseLink) -> Self {
        Self::spawn_inner(Some(link))
    }

    fn spawn_inner(link: Option<PulseLink>) -> Self {
        let (command_tx, command_rx) = unbounded_channel();
        let (event_tx, event_rx) = unbounded_channel();
        let (update_tx, update_rx) = channel();
        let thread = std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("tokio runtime");

            runtime.block_on(async move {
                let link = match link {
                    Some(link) => link,
                    None => match PulseLink::new().await {
                        Ok(link) => {
                            let _ = update_tx.send(Update::Log {
                                level: LogLevel::Info,
                                message: "BLE backend ready; press s or :scan v3".to_owned(),
                            });
                            link
                        }
                        Err(error) => {
                            let _ = update_tx.send(Update::Error(error.to_string()));
                            wait_for_shutdown(command_rx).await;
                            return;
                        }
                    },
                };
                run_worker(link, command_rx, event_tx, event_rx, update_tx).await;
            });
        });

        Self {
            commands: command_tx,
            updates: update_rx,
            thread,
        }
    }

    /// Sends a command to the worker.
    pub fn send(&self, command: Command) {
        let _ = self.commands.send(command);
    }

    /// Drains one update without blocking.
    #[must_use]
    pub fn try_recv(&self) -> Option<Update> {
        self.updates.try_recv().ok()
    }

    /// Waits for one update with a timeout.
    #[allow(dead_code)]
    pub fn recv_timeout(&self, timeout: Duration) -> Option<Update> {
        self.updates.recv_timeout(timeout).ok()
    }

    /// Requests shutdown.
    ///
    /// The worker is detached instead of joined because a V4 `wait_for_client`
    /// call may be blocked waiting for an app. The caller sends `StopAll` before
    /// this method; process exit then tears down the worker thread.
    pub fn shutdown(self) {
        let _ = self.commands.send(Command::Shutdown);
        let _ = self.thread;
    }
}

async fn wait_for_shutdown(mut commands: UnboundedReceiver<Command>) {
    while let Some(command) = commands.recv().await {
        if matches!(command, Command::Shutdown) {
            break;
        }
    }
}

struct WsSession {
    socket: Socket,
    client: AppClient,
}

enum WorkerEvent {
    WsReady {
        generation: u64,
        socket: Socket,
        client: AppClient,
    },
    WsFailed {
        generation: u64,
        message: String,
    },
}

struct WorkerState {
    link: PulseLink,
    devices: Vec<pulselink::transport::DiscoveredDevice>,
    session: Option<LocalSession>,
    event_tasks: Vec<TokioJoinHandle<()>>,
    ws: Option<WsSession>,
    ws_task: Option<TokioJoinHandle<()>>,
    ws_generation: u64,
    ws_target: Option<String>,
    relay: Option<LocalRelay>,
    events: UnboundedSender<WorkerEvent>,
    output_enabled: bool,
    max_intensity: u8,
    updates: Sender<Update>,
}

async fn run_worker(
    link: PulseLink,
    mut commands: UnboundedReceiver<Command>,
    event_tx: UnboundedSender<WorkerEvent>,
    mut event_rx: UnboundedReceiver<WorkerEvent>,
    updates: Sender<Update>,
) {
    let mut worker = WorkerState {
        link,
        devices: Vec::new(),
        session: None,
        event_tasks: Vec::new(),
        ws: None,
        ws_task: None,
        ws_generation: 0,
        ws_target: None,
        relay: None,
        events: event_tx,
        output_enabled: false,
        max_intensity: 20,
        updates,
    };

    loop {
        tokio::select! {
            command = commands.recv() => {
                let Some(command) = command else { break; };
                if matches!(command, Command::Shutdown) {
                    break;
                }
                worker.handle(command).await;
            }
            event = event_rx.recv() => {
                if let Some(event) = event {
                    worker.handle_event(event).await;
                }
            }
        }
    }
    worker.disconnect().await;
    worker.ws_disconnect().await;
    if let Some(relay) = worker.relay.take() {
        let _ = relay.shutdown().await;
    }
}

impl WorkerState {
    async fn handle(&mut self, command: Command) {
        match command {
            Command::Scan(kind) => self.scan(kind).await,
            Command::Connect(index) => self.connect(index).await,
            Command::Disconnect => self.disconnect().await,
            Command::V3Intensity(channel, value) => self.v3_intensity(channel, value).await,
            Command::V3Temp(channel, value, duration) => {
                self.v3_temp(channel, value, duration).await;
            }
            Command::V3PlayDemo => self.v3_play_demo().await,
            Command::V3Stop => self.v3_stop().await,
            Command::V3Reconnect => self.v3_reconnect().await,
            Command::V2Intensity(channel, value) => self.v2_intensity(channel, value).await,
            Command::V2Pulse(channel, x, y, z) => self.v2_pulse(channel, x, y, z).await,
            Command::OpossumIntensity(channel, value) => {
                self.opossum_intensity(channel, value).await;
            }
            Command::OpossumPulse(channel, frames) => {
                self.opossum_pulse(channel, frames).await;
            }
            Command::CivetStart => self.civet_start().await,
            Command::CivetStop => self.civet_stop().await,
            Command::CivetReset => self.civet_reset().await,
            Command::CivetOrientation(value) => self.civet_orientation(value).await,
            Command::PawNoTrigger => self.paw_no_trigger().await,
            Command::PawReset => self.paw_reset().await,
            Command::PawAngles => self.paw_angles().await,
            Command::PawLight(color) => self.paw_light(color).await,
            Command::WsConnect(url) => self.ws_connect(url).await,
            Command::WsLocal(bind) => self.ws_local(bind).await,
            Command::RelayStart(bind) => self.relay_start(bind).await,
            Command::RelayStop => self.relay_stop().await,
            Command::RelayStatus => self.relay_status(),
            Command::WsRefresh => self.ws_refresh().await,
            Command::WsIntensity {
                slot,
                channel,
                value,
            } => self.ws_intensity(&slot, channel, value).await,
            Command::WsReset { slot, channel } => self.ws_reset(&slot, channel).await,
            Command::WsClear => self.ws_clear().await,
            Command::SetOutput(enabled) => {
                self.output_enabled = enabled;
                self.log_info(format!(
                    "output {}",
                    if enabled { "enabled" } else { "disabled" }
                ));
            }
            Command::SetMaxIntensity(value) => {
                self.max_intensity = value.min(200);
                self.log_info(format!("max intensity set to {}", self.max_intensity));
            }
            Command::StopAll => self.stop_all().await,
            Command::Shutdown => {}
        }
    }

    async fn scan(&mut self, kind: Option<DeviceKind>) {
        let result = if let Some(kind) = kind {
            if let Some(profile) = profile_for(&kind) {
                self.link
                    .scan_profile(profile, Duration::from_secs(5))
                    .await
            } else {
                self.log_error("device profile is not available");
                return;
            }
        } else {
            let all = ScanFilter::default();
            match self.link.scan(all, Duration::from_secs(5)).await {
                Ok(devices) if !devices.is_empty() => Ok(devices),
                Ok(_) | Err(_) => {
                    self.log_info(
                        "empty/failed unfiltered scan; retrying with 0x180B/0x180C service filters"
                            .to_owned(),
                    );
                    let filtered = ScanFilter {
                        name_prefixes: Vec::new(),
                        service_uuids: vec![SERVICE_PWM, SERVICE_CONTROL],
                    };
                    self.link.scan(filtered, Duration::from_secs(5)).await
                }
            }
        };
        match result {
            Ok(devices) => {
                if devices.is_empty() {
                    self.log_info(
                        "no devices found; close the DG-LAB app, wake the device, or try `:scan v3`"
                            .to_owned(),
                    );
                } else {
                    self.log_info(format!("scan found {} device(s)", devices.len()));
                    if devices.iter().any(|device| {
                        device.name.is_none() && device.service_uuids.contains(&SERVICE_CONTROL)
                    }) {
                        self.log_info(
                            "found unnamed 0x180C device; possible Coyote V3 privacy broadcast. \
                             Connect by index, or switch to public broadcast: power on, push both \
                             wheels down and hold until the main LED pattern changes."
                                .to_owned(),
                        );
                    }
                }
                self.devices.clone_from(&devices);
                let _ = self.updates.send(Update::Devices(devices));
            }
            Err(error) => self.log_error(error.to_string()),
        }
    }

    async fn connect(&mut self, index: usize) {
        let Some(device) = self.devices.get(index).cloned() else {
            self.log_error(format!("no scan result at index {index}"));
            return;
        };
        self.disconnect().await;
        match self.link.connect_device(&device).await {
            Ok(session) => {
                let kind = session.kind();
                let state = session.state();
                self.spawn_events(&session);
                self.session = Some(session);
                self.log_info(format!("connected {kind:?}"));
                let _ = self.updates.send(Update::Connected { kind, state });
            }
            Err(error) => self.log_error(error.to_string()),
        }
    }

    fn spawn_events(&mut self, session: &LocalSession) {
        match session {
            LocalSession::CoyoteV3(session) => {
                let mut receiver = session.subscribe();
                let updates = self.updates.clone();
                let state = session.clone();
                self.event_tasks.push(tokio::spawn(async move {
                    while let Ok(event) = receiver.recv().await {
                        let _ = updates.send(Update::V3Event(event));
                        let _ = updates.send(Update::V3State(state.state()));
                    }
                }));
            }
            LocalSession::CoyoteV2(session) => {
                self.spawn_local_events(
                    LocalSession::CoyoteV2(session.clone()),
                    session.subscribe(),
                );
            }
            LocalSession::Opossum(session) => {
                self.spawn_local_events(
                    LocalSession::Opossum(session.clone()),
                    session.subscribe(),
                );
            }
            LocalSession::Civet(session) => {
                self.spawn_local_events(LocalSession::Civet(session.clone()), session.subscribe());
            }
            LocalSession::PawPrints(session) => {
                self.spawn_local_events(
                    LocalSession::PawPrints(session.clone()),
                    session.subscribe(),
                );
            }
            _ => {}
        }
    }

    fn spawn_local_events(
        &mut self,
        session: LocalSession,
        mut receiver: tokio::sync::broadcast::Receiver<LocalEvent>,
    ) {
        let updates = self.updates.clone();
        self.event_tasks.push(tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                let _ = updates.send(Update::LocalEvent(event));
                let _ = updates.send(Update::State(session.state()));
            }
        }));
    }

    async fn disconnect(&mut self) {
        for task in self.event_tasks.drain(..) {
            task.abort();
        }
        if let Some(session) = self.session.take() {
            let _ = session.shutdown().await;
            let _ = self.updates.send(Update::Disconnected);
        }
    }

    async fn ws_disconnect(&mut self) {
        self.ws_generation = self.ws_generation.wrapping_add(1);
        if let Some(task) = self.ws_task.take() {
            task.abort();
        }
        if let Some(ws) = self.ws.take() {
            let _ = ws.socket.shutdown().await;
        }
        self.ws_target = None;
        let _ = self.updates.send(Update::WsState {
            target: None,
            devices: Vec::new(),
        });
    }

    fn output_allowed(&self, intensity: u8) -> bool {
        if intensity == 0 {
            return true;
        }
        if !self.output_enabled {
            self.log_error("output is disabled; press Space or run `output on`");
            return false;
        }
        if intensity > self.max_intensity {
            self.log_error(format!(
                "intensity {intensity} exceeds lab limit {}",
                self.max_intensity
            ));
            return false;
        }
        true
    }

    async fn v3_intensity(&mut self, channel: Channel, value: u8) {
        if !self.output_allowed(value) {
            return;
        }
        let Some(LocalSession::CoyoteV3(session)) = &self.session else {
            self.log_error("no Coyote V3 session");
            return;
        };
        match Intensity::new(value) {
            Ok(intensity) => match session.set_intensity(channel, intensity).await {
                Ok(change) => self.log_info(format!("V3 intensity: {change:?}")),
                Err(error) => self.log_error(error.to_string()),
            },
            Err(error) => self.log_error(error.to_string()),
        }
    }

    async fn v3_temp(&mut self, channel: Channel, value: u8, duration: u64) {
        if !self.output_allowed(value) {
            return;
        }
        let Some(LocalSession::CoyoteV3(session)) = &self.session else {
            self.log_error("no Coyote V3 session");
            return;
        };
        let intensity = match Intensity::new(value) {
            Ok(value) => value,
            Err(error) => {
                self.log_error(error.to_string());
                return;
            }
        };
        match session.set_intensity(channel, intensity).await {
            Ok(change) => {
                self.log_info(format!("V3 temp start: {change:?}"));
                tokio::time::sleep(Duration::from_millis(duration)).await;
                if let Err(error) = session.set_intensity(channel, Intensity::MIN).await {
                    self.log_error(error.to_string());
                }
            }
            Err(error) => self.log_error(error.to_string()),
        }
    }

    async fn v3_play_demo(&mut self) {
        if !self.output_allowed(1) {
            return;
        }
        let Some(LocalSession::CoyoteV3(session)) = &self.session else {
            self.log_error("no Coyote V3 session");
            return;
        };
        let Ok(interval) = PulseInterval::new(100) else {
            return;
        };
        let Ok(intensity) = PulseIntensity::new(2) else {
            return;
        };
        let sample = PulseSample::new(interval, intensity);
        let frame = StereoFrame::new(ChannelFrame::Active([sample; 4]), ChannelFrame::Silent);
        let Ok(pattern) = Pattern::new(vec![frame; 5], true) else {
            return;
        };
        match session.play(pattern, PlayOptions::default()).await {
            Ok(_) => self.log_info("V3 demo pattern started".to_owned()),
            Err(error) => self.log_error(error.to_string()),
        }
    }

    async fn v3_stop(&mut self) {
        let Some(LocalSession::CoyoteV3(session)) = &self.session else {
            self.log_error("no Coyote V3 session");
            return;
        };
        match session.stop().await {
            Ok(outcome) => self.log_info(format!("V3 stop: {outcome:?}")),
            Err(error) => self.log_error(error.to_string()),
        }
    }

    async fn v3_reconnect(&mut self) {
        let Some(LocalSession::CoyoteV3(session)) = &self.session else {
            self.log_error("no Coyote V3 session");
            return;
        };
        match session.reconnect().await {
            Ok(()) => self.log_info("V3 reconnect requested".to_owned()),
            Err(error) => self.log_error(error.to_string()),
        }
    }

    async fn v2_intensity(&mut self, channel: Channel, value: u16) {
        let Some(LocalSession::CoyoteV2(session)) = &self.session else {
            self.log_error("no Coyote V2 session");
            return;
        };
        let Ok(intensity) = CoyoteV2Intensity::new(value) else {
            self.log_error("V2 intensity must be 0..=2047".to_owned());
            return;
        };
        if !self.output_allowed(intensity.display()) {
            return;
        }
        match session.set_intensity(channel, intensity).await {
            Ok(()) => self.log_info(format!("V2 intensity {channel:?}={value}")),
            Err(error) => self.log_error(error.to_string()),
        }
    }

    async fn v2_pulse(&mut self, channel: Channel, x: u8, y: u16, z: u8) {
        let Some(LocalSession::CoyoteV2(session)) = &self.session else {
            self.log_error("no Coyote V2 session");
            return;
        };
        let Ok(pulse) = PwmPulse::new(x, y, z) else {
            self.log_error("invalid V2 pulse".to_owned());
            return;
        };
        match session.set_pulse(channel, pulse).await {
            Ok(()) => self.log_info(format!("V2 pulse {channel:?}=[{x},{y},{z}]")),
            Err(error) => self.log_error(error.to_string()),
        }
    }

    async fn opossum_intensity(&mut self, channel: Channel, value: u8) {
        if !self.output_allowed(value) {
            return;
        }
        let Some(LocalSession::Opossum(session)) = &self.session else {
            self.log_error("no Opossum session");
            return;
        };
        let intensity = match Intensity::new(value) {
            Ok(value) => value,
            Err(error) => {
                self.log_error(error.to_string());
                return;
            }
        };
        match session.set_intensity(channel, intensity).await {
            Ok(()) => self.log_info(format!("Opossum intensity {channel:?}={value}")),
            Err(error) => self.log_error(error.to_string()),
        }
    }

    async fn opossum_pulse(&mut self, channel: Channel, frames: [u8; 4]) {
        if !self.output_allowed(*frames.iter().max().unwrap_or(&0)) {
            return;
        }
        let Some(LocalSession::Opossum(session)) = &self.session else {
            self.log_error("no Opossum session");
            return;
        };
        match session.set_pulse(channel, frames).await {
            Ok(()) => self.log_info(format!("Opossum pulse {channel:?}={frames:?}")),
            Err(error) => self.log_error(error.to_string()),
        }
    }

    async fn civet_start(&mut self) {
        if let Some(LocalSession::Civet(session)) = &self.session {
            match session.start_pressure().await {
                Ok(()) => self.log_info("Civet pressure started".to_owned()),
                Err(error) => self.log_error(error.to_string()),
            }
        }
    }

    async fn civet_stop(&mut self) {
        if let Some(LocalSession::Civet(session)) = &self.session {
            match session.stop_pressure().await {
                Ok(()) => self.log_info("Civet pressure stopped".to_owned()),
                Err(error) => self.log_error(error.to_string()),
            }
        }
    }

    async fn civet_reset(&mut self) {
        if let Some(LocalSession::Civet(session)) = &self.session {
            match session.reset_pressure().await {
                Ok(()) => self.log_info("Civet pressure reset".to_owned()),
                Err(error) => self.log_error(error.to_string()),
            }
        }
    }

    async fn civet_orientation(&mut self, value: u8) {
        if let Some(LocalSession::Civet(session)) = &self.session {
            match session.set_orientation(value).await {
                Ok(()) => self.log_info(format!("Civet orientation {value}")),
                Err(error) => self.log_error(error.to_string()),
            }
        }
    }

    async fn paw_no_trigger(&mut self) {
        if let Some(LocalSession::PawPrints(session)) = &self.session {
            match session.configure(Command50::no_trigger(1)).await {
                Ok(()) => self.log_info("PawPrints no-trigger configured".to_owned()),
                Err(error) => self.log_error(error.to_string()),
            }
        }
    }

    async fn paw_reset(&mut self) {
        if let Some(LocalSession::PawPrints(session)) = &self.session {
            match session.reset_parameter().await {
                Ok(()) => self.log_info("PawPrints parameter reset".to_owned()),
                Err(error) => self.log_error(error.to_string()),
            }
        }
    }

    async fn paw_angles(&mut self) {
        if let Some(LocalSession::PawPrints(session)) = &self.session {
            match session.auto_detect_angles().await {
                Ok(()) => self.log_info("PawPrints angle detection started".to_owned()),
                Err(error) => self.log_error(error.to_string()),
            }
        }
    }

    async fn paw_light(&mut self, color: u8) {
        if let Some(LocalSession::PawPrints(session)) = &self.session {
            match session.set_shoulder_light(color).await {
                Ok(()) => self.log_info(format!("PawPrints light {color}")),
                Err(error) => self.log_error(error.to_string()),
            }
        }
    }

    async fn ws_connect(&mut self, url: String) {
        self.ws_disconnect().await;
        let is_local = self
            .relay
            .as_ref()
            .is_some_and(|relay| relay.controller_url() == url);
        let socket = match Socket::connect(url).await {
            Ok(socket) => socket,
            Err(error) => {
                self.log_error(error.to_string());
                return;
            }
        };

        let target = socket.target_id().to_owned();
        self.ws_target = Some(target.clone());
        self.log_info(format!("V4 target id: {target}"));
        if is_local && let Some(relay) = &self.relay {
            self.log_info(format!("pairing URL: {}", relay.pairing_url(&target)));
            self.log_info(format!("QR URL: {}", relay.qr_url(&target)));
        }
        let _ = self.updates.send(Update::WsState {
            target: Some(target),
            devices: Vec::new(),
        });

        self.ws_generation = self.ws_generation.wrapping_add(1);
        let generation = self.ws_generation;
        let events = self.events.clone();
        let task = tokio::spawn(async move {
            match timeout(Duration::from_secs(300), socket.wait_for_client()).await {
                Ok(Ok(client)) => {
                    let _ = events.send(WorkerEvent::WsReady {
                        generation,
                        socket,
                        client,
                    });
                }
                Ok(Err(error)) => {
                    let _ = events.send(WorkerEvent::WsFailed {
                        generation,
                        message: error.to_string(),
                    });
                }
                Err(_) => {
                    let _ = events.send(WorkerEvent::WsFailed {
                        generation,
                        message: "timed out waiting for V4 app".to_owned(),
                    });
                }
            }
        });
        self.ws_task = Some(task);
    }

    async fn ws_local(&mut self, bind: Option<std::net::SocketAddr>) {
        self.ws_disconnect().await;
        if self.relay.is_none() {
            self.relay_start(bind).await;
        }
        let Some(url) = self.relay.as_ref().map(LocalRelay::controller_url) else {
            return;
        };
        self.log_info(format!("local relay: {url}"));
        self.ws_connect(url).await;
    }

    async fn relay_start(&mut self, bind: Option<std::net::SocketAddr>) {
        if self.relay.is_some() {
            self.log_error("local relay is already running".to_owned());
            return;
        }
        let default = RelayConfig::default();
        let config = RelayConfig {
            bind: bind.unwrap_or(default.bind),
            ..default
        };
        match LocalRelay::start(config).await {
            Ok(relay) => {
                self.log_info(format!("local relay listening: {}", relay.controller_url()));
                self.relay = Some(relay);
            }
            Err(error) => self.log_error(error.to_string()),
        }
    }

    async fn relay_stop(&mut self) {
        self.ws_disconnect().await;
        if let Some(relay) = self.relay.take() {
            match relay.shutdown().await {
                Ok(()) => self.log_info("local relay stopped".to_owned()),
                Err(error) => self.log_error(error.to_string()),
            }
        } else {
            self.log_error("local relay is not running".to_owned());
        }
    }

    fn relay_status(&self) {
        if let Some(relay) = &self.relay {
            self.log_info(format!("local relay: {}", relay.controller_url()));
        } else {
            self.log_info("local relay is not running".to_owned());
        }
    }

    async fn handle_event(&mut self, event: WorkerEvent) {
        match event {
            WorkerEvent::WsReady {
                generation,
                socket,
                client,
            } => {
                if generation != self.ws_generation {
                    let _ = socket.shutdown().await;
                    return;
                }
                self.ws_task = None;
                let devices = self.ws_snapshots(&client).await;
                let _ = self.updates.send(Update::WsState {
                    target: self.ws_target.clone(),
                    devices,
                });
                self.log_info("V4 app connected".to_owned());
                self.ws = Some(WsSession { socket, client });
            }
            WorkerEvent::WsFailed {
                generation,
                message,
            } => {
                if generation == self.ws_generation {
                    self.ws_task = None;
                    self.log_error(message);
                }
            }
        }
    }

    async fn ws_refresh(&mut self) {
        let Some(ws) = &self.ws else {
            self.log_error("V4 is not connected");
            return;
        };
        let devices = self.ws_snapshots(&ws.client).await;
        let target = Some(ws.socket.target_id().to_owned());
        let _ = self.updates.send(Update::WsState { target, devices });
    }

    async fn ws_snapshots(&self, client: &AppClient) -> Vec<RemoteDeviceSnapshot> {
        if let Err(error) = client.request_devices().await {
            self.log_error(error.to_string());
            return Vec::new();
        }
        client
            .devices()
            .into_iter()
            .filter_map(|device| client.device_snapshot(&device.slot_id))
            .collect()
    }

    async fn ws_intensity(&mut self, slot: &str, channel: Channel, value: u8) {
        if !self.output_allowed(value) {
            return;
        }
        let Some(ws) = &self.ws else {
            self.log_error("V4 is not connected");
            return;
        };
        let Some(handle) = ws.client.device(slot) else {
            self.log_error(format!("V4 slot not found: {slot}"));
            return;
        };
        let Ok(intensity) = Intensity::new(value) else {
            self.log_error("intensity must be 0..=200".to_owned());
            return;
        };
        let options = OperationOptions::default().with_timeout(Duration::from_secs(5));
        let result = handle
            .set_temp_intensity(channel, intensity, Duration::from_millis(500), options)
            .await;
        if let Err(error) = result {
            self.log_error(error.to_string());
        }
    }

    async fn ws_reset(&mut self, slot: &str, channel: Channel) {
        let Some(ws) = &self.ws else {
            self.log_error("V4 is not connected");
            return;
        };
        let Some(handle) = ws.client.device(slot) else {
            self.log_error(format!("V4 slot not found: {slot}"));
            return;
        };
        let options = OperationOptions::default().with_timeout(Duration::from_secs(5));
        if let Err(error) = handle.reset_intensity(channel, options).await {
            self.log_error(error.to_string());
        }
    }

    async fn ws_clear(&mut self) {
        let Some(ws) = &self.ws else {
            self.log_error("V4 is not connected");
            return;
        };
        if let Err(error) = ws.client.clear_operate(ClearTarget::All).await {
            self.log_error(error.to_string());
        }
    }

    async fn stop_all(&mut self) {
        self.log_info("emergency stop requested".to_owned());
        if let Some(session) = &self.session {
            match session {
                LocalSession::CoyoteV3(session) => {
                    let _ = session.stop().await;
                }
                LocalSession::CoyoteV2(session) => {
                    let _ = session
                        .set_intensity(Channel::A, CoyoteV2Intensity::MIN)
                        .await;
                    let _ = session
                        .set_intensity(Channel::B, CoyoteV2Intensity::MIN)
                        .await;
                }
                LocalSession::Opossum(session) => {
                    let _ = session.set_intensity(Channel::A, Intensity::MIN).await;
                    let _ = session.set_intensity(Channel::B, Intensity::MIN).await;
                    let _ = session.set_pulse(Channel::A, [0; 4]).await;
                    let _ = session.set_pulse(Channel::B, [0; 4]).await;
                }
                LocalSession::Civet(session) => {
                    let _ = session.stop_pressure().await;
                }
                _ => {}
            }
        }
        if let Some(ws) = &self.ws {
            let _ = ws.client.clear_operate(ClearTarget::All).await;
        }
    }

    fn log_info(&self, message: impl Into<String>) {
        let _ = self.updates.send(Update::Log {
            level: LogLevel::Info,
            message: message.into(),
        });
    }

    fn log_error(&self, message: impl Into<String>) {
        let _ = self.updates.send(Update::Error(message.into()));
    }
}
