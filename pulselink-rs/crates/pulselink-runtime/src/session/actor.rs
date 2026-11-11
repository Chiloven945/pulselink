//! Session actor state and main loop.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use pulselink_core::{ChannelMap, ConnectionState, Intensity, Observed};
use pulselink_transport::{Backend, GattLink, ReconnectTarget};
use tokio::sync::{broadcast, mpsc, oneshot, watch};
use tokio::task::JoinHandle;
use tokio::time::{Instant, Interval, interval_at, sleep_until};
use tokio_util::sync::CancellationToken;

use crate::config::CoyoteV3Config;
use crate::error::RuntimeError;
use crate::event::CoyoteV3Event;
use crate::sequence::SequenceAllocator;
use crate::state::{CoyoteV3Settings, CoyoteV3State, PatternId, PlayEndReason, PlaybackState};

use super::ack::{IntensityRequest, PendingIntensity};
use super::command::SessionCommand;
use super::notification::Notification;
use super::playback::Playback;
use super::reconnect::ReconnectResult;

pub(super) struct ActorChannels {
    pub(super) state_tx: watch::Sender<CoyoteV3State>,
    pub(super) events: broadcast::Sender<CoyoteV3Event>,
    pub(super) commands: mpsc::Receiver<SessionCommand>,
    pub(super) playback_tx: watch::Sender<PlaybackState>,
    pub(super) playback_ends: Arc<Mutex<HashMap<PatternId, PlayEndReason>>>,
    pub(super) cancel: CancellationToken,
}

pub(super) struct Actor {
    pub(super) config: CoyoteV3Config,
    pub(super) backend: Arc<dyn Backend>,
    pub(super) reconnect_target: ReconnectTarget,
    pub(super) state: CoyoteV3State,
    pub(super) state_tx: watch::Sender<CoyoteV3State>,
    pub(super) events: broadcast::Sender<CoyoteV3Event>,
    pub(super) commands: mpsc::Receiver<SessionCommand>,
    pub(super) playback_tx: watch::Sender<PlaybackState>,
    pub(super) playback_ends: Arc<Mutex<HashMap<PatternId, PlayEndReason>>>,
    pub(super) cancel: CancellationToken,
    pub(super) link: Option<Box<dyn GattLink>>,
    pub(super) notification_tx: mpsc::Sender<Notification>,
    pub(super) notification_rx: mpsc::Receiver<Notification>,
    pub(super) notification_task: Option<JoinHandle<()>>,
    pub(super) reconnect_tx: mpsc::Sender<ReconnectResult>,
    pub(super) reconnect_rx: mpsc::Receiver<ReconnectResult>,
    pub(super) reconnect_task: Option<JoinHandle<()>>,
    pub(super) reconnect_reply: Option<oneshot::Sender<Result<(), RuntimeError>>>,
    pub(super) pending: Option<PendingIntensity>,
    pub(super) queue: VecDeque<IntensityRequest>,
    pub(super) sequence: SequenceAllocator,
    pub(super) playground: Option<Playback>,
    pub(super) next_pattern_id: u64,
    pub(super) desired: ChannelMap<Observed<Intensity>>,
    pub(super) settings: Option<CoyoteV3Settings>,
    pub(super) ticker: Interval,
    pub(super) ticker_active: bool,
    pub(super) last_tick: Instant,
    pub(super) next_deadline: Option<Instant>,
    pub(super) closed: bool,
}

impl Actor {
    pub(super) fn new(
        config: CoyoteV3Config,
        backend: Arc<dyn Backend>,
        reconnect_target: ReconnectTarget,
        channels: ActorChannels,
    ) -> Self {
        let ActorChannels {
            state_tx,
            events,
            commands,
            playback_tx,
            playback_ends,
            cancel,
        } = channels;
        let (notification_tx, notification_rx) = mpsc::channel(32);
        let (reconnect_tx, reconnect_rx) = mpsc::channel(8);
        let state = state_tx.borrow().clone();
        let ticker = interval_at(Instant::now() + config.tick_interval, config.tick_interval);
        Self {
            config,
            backend,
            reconnect_target,
            state,
            state_tx,
            events,
            commands,
            playback_tx,
            playback_ends,
            cancel,
            link: None,
            notification_tx,
            notification_rx,
            notification_task: None,
            reconnect_tx,
            reconnect_rx,
            reconnect_task: None,
            reconnect_reply: None,
            pending: None,
            queue: VecDeque::new(),
            sequence: SequenceAllocator::new(),
            playground: None,
            next_pattern_id: 1,
            desired: ChannelMap::new(Observed::Unknown, Observed::Unknown),
            settings: None,
            ticker,
            ticker_active: false,
            last_tick: Instant::now(),
            next_deadline: None,
            closed: false,
        }
    }

    pub(super) async fn run(
        mut self,
        link: Box<dyn GattLink>,
        init: oneshot::Sender<Result<(), RuntimeError>>,
    ) {
        match self.initialize(link).await {
            Ok(()) => {
                let _ = init.send(Ok(()));
            }
            Err(error) => {
                self.state.common.connection = ConnectionState::Disconnected;
                self.publish();
                let _ = init.send(Err(error));
                return;
            }
        }

        loop {
            let deadline = self.next_deadline;
            tokio::select! {
                biased;

                () = self.cancel.cancelled() => {
                    self.shutdown_actor().await;
                    break;
                }

                command = self.commands.recv() => {
                    let Some(command) = command else {
                        self.shutdown_actor().await;
                        break;
                    };
                    if self.handle_command(command).await {
                        break;
                    }
                }

                notification = self.notification_rx.recv() => {
                    match notification {
                        Some(notification) => self.handle_notification(notification).await,
                        None => self.link_lost("notification task ended".to_owned()).await,
                    }
                }

                _ = self.ticker.tick(), if self.ticker_active => {
                    self.handle_tick().await;
                }

                () = wait_deadline(deadline), if deadline.is_some() => {
                    self.handle_deadline();
                }

                result = self.reconnect_rx.recv(), if self.reconnect_task.is_some() => {
                    if let Some(result) = result {
                        self.handle_reconnect_result(result).await;
                    }
                }
            }
        }
    }

    pub(super) fn publish(&mut self) {
        self.state.common.touch();
        let _ = self.state_tx.send(self.state.clone());
    }

    pub(super) fn emit(&self, event: CoyoteV3Event) {
        let _ = self.events.send(event);
    }
}

async fn wait_deadline(deadline: Option<Instant>) {
    match deadline {
        Some(deadline) => sleep_until(deadline).await,
        None => std::future::pending().await,
    }
}
