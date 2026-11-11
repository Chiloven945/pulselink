//! Public session and playback handles.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use pulselink_core::{
    Channel, ConnectionState, DeviceId, DeviceInfo, DeviceState, Intensity, IntensityDelta,
    Observed,
};
use pulselink_pattern::Pattern;
use pulselink_transport::{Backend, GattLink, ReconnectTarget, TransportError};
use tokio::sync::{broadcast, mpsc, oneshot, watch};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

use crate::config::{CoyoteV3Balance, CoyoteV3Config};
use crate::error::RuntimeError;
use crate::event::CoyoteV3Event;
use crate::state::{
    CoyoteV3State, IntensityChange, PatternId, PlayEndReason, PlayOptions, PlaybackState,
    StopOutcome,
};

use super::actor::{Actor, ActorChannels};
use super::command::SessionCommand;

struct SessionHandle {
    id: DeviceId,
    info: DeviceInfo,
    config: CoyoteV3Config,
    commands: mpsc::Sender<SessionCommand>,
    state_rx: watch::Receiver<CoyoteV3State>,
    events: broadcast::Sender<CoyoteV3Event>,
    playback_rx: watch::Receiver<PlaybackState>,
    playback_ends: Arc<Mutex<HashMap<PatternId, PlayEndReason>>>,
    cancel: CancellationToken,
    task: Mutex<Option<JoinHandle<()>>>,
}

/// A cloneable handle to one initialized Coyote V3 session.
#[derive(Clone)]
pub struct CoyoteV3Session {
    inner: Arc<SessionHandle>,
}

pub(crate) struct SessionStart {
    pub(crate) backend: Arc<dyn Backend>,
    pub(crate) link: Box<dyn GattLink>,
    pub(crate) info: DeviceInfo,
    pub(crate) config: CoyoteV3Config,
    pub(crate) reconnect_target: ReconnectTarget,
    pub(crate) init_timeout: Duration,
    pub(crate) command_capacity: usize,
    pub(crate) event_capacity: usize,
}

impl CoyoteV3Session {
    pub(crate) async fn start(params: SessionStart) -> Result<Self, RuntimeError> {
        let SessionStart {
            backend,
            link,
            info,
            config,
            reconnect_target,
            init_timeout,
            command_capacity,
            event_capacity,
        } = params;
        let (command_tx, command_rx) = mpsc::channel(command_capacity.max(1));
        let initial = CoyoteV3State {
            common: DeviceState::new(ConnectionState::Initializing),
            settings: Observed::Unknown,
            pattern: None,
            last_b1: None,
        };
        let (state_tx, state_rx) = watch::channel(initial);
        let (events, _) = broadcast::channel(event_capacity.max(1));
        let (playback_tx, playback_rx) = watch::channel(PlaybackState::Idle);
        let playback_ends = Arc::new(Mutex::new(HashMap::new()));
        let cancel = CancellationToken::new();
        let (init_tx, init_rx) = oneshot::channel();

        let actor = Actor::new(
            config.clone(),
            backend,
            reconnect_target,
            ActorChannels {
                state_tx,
                events: events.clone(),
                commands: command_rx,
                playback_tx,
                playback_ends: Arc::clone(&playback_ends),
                cancel: cancel.clone(),
            },
        );
        let task = tokio::spawn(actor.run(link, init_tx));

        match timeout(init_timeout, init_rx).await {
            Ok(Ok(Ok(()))) => Ok(Self {
                inner: Arc::new(SessionHandle {
                    id: info.id.clone(),
                    info,
                    config,
                    commands: command_tx,
                    state_rx,
                    events,
                    playback_rx,
                    playback_ends,
                    cancel,
                    task: Mutex::new(Some(task)),
                }),
            }),
            Ok(Ok(Err(error))) => {
                task.abort();
                Err(error)
            }
            Ok(Err(_)) => {
                task.abort();
                Err(RuntimeError::Closed)
            }
            Err(_) => {
                cancel.cancel();
                task.abort();
                Err(RuntimeError::Transport(TransportError::Timeout))
            }
        }
    }

    /// Returns the device id.
    #[must_use]
    pub fn id(&self) -> &DeviceId {
        &self.inner.id
    }

    /// Returns descriptive device information.
    #[must_use]
    pub fn device_info(&self) -> &DeviceInfo {
        &self.inner.info
    }

    /// Returns the latest state snapshot without blocking.
    #[must_use]
    pub fn state(&self) -> CoyoteV3State {
        self.inner.state_rx.borrow().clone()
    }

    /// Subscribes to session events.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<CoyoteV3Event> {
        self.inner.events.subscribe()
    }

    /// Sets an absolute channel intensity and waits for the B1 acknowledgement.
    pub async fn set_intensity(
        &self,
        channel: Channel,
        intensity: Intensity,
    ) -> Result<IntensityChange, RuntimeError> {
        let (reply, receiver) = oneshot::channel();
        self.send(SessionCommand::SetIntensity {
            channel,
            intensity,
            reply,
        })?;
        receiver.await.map_err(|_| RuntimeError::Closed)?
    }

    /// Adjusts a channel intensity by a relative delta and waits for B1.
    pub async fn adjust_intensity(
        &self,
        channel: Channel,
        delta: IntensityDelta,
    ) -> Result<IntensityChange, RuntimeError> {
        let (reply, receiver) = oneshot::channel();
        self.send(SessionCommand::AdjustIntensity {
            channel,
            delta,
            reply,
        })?;
        receiver.await.map_err(|_| RuntimeError::Closed)?
    }

    /// Updates a per-channel BF soft limit.
    pub async fn set_soft_limit(
        &self,
        channel: Channel,
        limit: Intensity,
    ) -> Result<(), RuntimeError> {
        let (reply, receiver) = oneshot::channel();
        self.send(SessionCommand::SetSoftLimit {
            channel,
            limit,
            reply,
        })?;
        receiver.await.map_err(|_| RuntimeError::Closed)?
    }

    /// Updates the BF frequency/intensity balance.
    pub async fn set_balance(&self, balance: CoyoteV3Balance) -> Result<(), RuntimeError> {
        let (reply, receiver) = oneshot::channel();
        self.send(SessionCommand::SetBalance { balance, reply })?;
        receiver.await.map_err(|_| RuntimeError::Closed)?
    }

    /// Starts a pattern. The returned handle tracks completion.
    pub async fn play(
        &self,
        pattern: Pattern,
        options: PlayOptions,
    ) -> Result<PlayHandle, RuntimeError> {
        let (reply, receiver) = oneshot::channel();
        self.send(SessionCommand::Play {
            pattern,
            options,
            reply,
        })?;
        let id = receiver.await.map_err(|_| RuntimeError::Closed)??;
        Ok(PlayHandle {
            id,
            state: self.inner.playback_rx.clone(),
            ends: Arc::clone(&self.inner.playback_ends),
            commands: self.inner.commands.clone(),
        })
    }

    /// Stops playback and sends an absolute zero-intensity B0.
    pub async fn stop(&self) -> Result<StopOutcome, RuntimeError> {
        let (reply, receiver) = oneshot::channel();
        self.send(SessionCommand::Stop { reply })?;
        receiver.await.map_err(|_| RuntimeError::Closed)?
    }

    /// Reconnects the session if it is not ready.
    pub async fn reconnect(&self) -> Result<(), RuntimeError> {
        let (reply, receiver) = oneshot::channel();
        self.send(SessionCommand::Reconnect { reply })?;
        receiver.await.map_err(|_| RuntimeError::Closed)?
    }

    /// Stops all work, attempts a final zero-intensity write, and closes the link.
    pub async fn shutdown(self) -> Result<(), RuntimeError> {
        let (reply, receiver) = oneshot::channel();
        match self
            .inner
            .commands
            .try_send(SessionCommand::Shutdown { reply })
        {
            Ok(()) => {}
            Err(mpsc::error::TrySendError::Closed(_)) => return Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => return Err(RuntimeError::Busy),
        }
        let result = match timeout(
            self.inner.config.ack_timeout + Duration::from_secs(1),
            receiver,
        )
        .await
        {
            Ok(reply) => reply.map_err(|_| RuntimeError::Closed)?,
            Err(_) => Err(RuntimeError::Closed),
        };
        self.inner.cancel.cancel();
        let task = self
            .inner
            .task
            .lock()
            .expect("session task mutex poisoned")
            .take();
        if let Some(task) = task {
            let _ = timeout(Duration::from_secs(2), task).await;
        }
        result
    }

    fn send(&self, command: SessionCommand) -> Result<(), RuntimeError> {
        match self.inner.commands.try_send(command) {
            Ok(()) => Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => Err(RuntimeError::Busy),
            Err(mpsc::error::TrySendError::Closed(_)) => Err(RuntimeError::Closed),
        }
    }
}

impl Drop for SessionHandle {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

/// A handle to one pattern playback.
pub struct PlayHandle {
    id: PatternId,
    state: watch::Receiver<PlaybackState>,
    ends: Arc<Mutex<HashMap<PatternId, PlayEndReason>>>,
    commands: mpsc::Sender<SessionCommand>,
}

impl PlayHandle {
    /// Returns the pattern id.
    #[must_use]
    pub const fn id(&self) -> PatternId {
        self.id
    }

    /// Returns whether playback has ended.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.ends
            .lock()
            .expect("playback ends mutex poisoned")
            .contains_key(&self.id)
            || self.state.borrow().is_finished()
    }

    /// Waits for playback to finish.
    pub async fn wait(mut self) -> Result<PlayEndReason, RuntimeError> {
        loop {
            if let Some(reason) = self
                .ends
                .lock()
                .expect("playback ends mutex poisoned")
                .get(&self.id)
                .copied()
            {
                return Ok(reason);
            }
            self.state
                .changed()
                .await
                .map_err(|_| RuntimeError::Closed)?;
        }
    }

    /// Stops this playback without changing intensity.
    pub async fn stop(self) -> Result<(), RuntimeError> {
        let (reply, receiver) = oneshot::channel();
        match self
            .commands
            .try_send(SessionCommand::StopPlayback { id: self.id, reply })
        {
            Ok(()) => receiver.await.map_err(|_| RuntimeError::Closed)?,
            Err(mpsc::error::TrySendError::Full(_)) => Err(RuntimeError::Busy),
            Err(mpsc::error::TrySendError::Closed(_)) => Err(RuntimeError::Closed),
        }
    }
}
