//! Live remote-device handles and device operations.

use pulselink_core::{Channel, DeviceKind};

use crate::app::AppClient;
use crate::error::WsError;
use crate::operation::{OperationHandle, OperationOptions, PulseFrame};

use super::snapshot::RemoteDeviceSnapshot;
use super::state::SharedState;
use super::typed::TypedRemoteSnapshot;

/// A live handle to one remote device.
#[derive(Clone)]
pub struct RemoteDeviceHandle {
    client: AppClient,
    state: SharedState,
    slot_id: String,
}

impl RemoteDeviceHandle {
    pub(crate) fn new(client: AppClient, state: SharedState, slot_id: String) -> Self {
        Self {
            client,
            state,
            slot_id,
        }
    }

    /// Returns the device slot id.
    #[must_use]
    pub fn slot_id(&self) -> &str {
        &self.slot_id
    }

    /// Returns the current display name.
    #[must_use]
    pub fn name(&self) -> String {
        self.state
            .read()
            .expect("device state lock poisoned")
            .name
            .clone()
    }

    /// Returns the current device type.
    #[must_use]
    pub fn device_type(&self) -> String {
        self.state
            .read()
            .expect("device state lock poisoned")
            .device_type
            .clone()
    }

    /// Returns the mapped device kind.
    #[must_use]
    pub fn kind(&self) -> DeviceKind {
        self.snapshot().kind()
    }

    /// Returns a typed snapshot view.
    #[must_use]
    pub fn typed_snapshot(&self) -> TypedRemoteSnapshot {
        self.snapshot().typed()
    }

    /// Returns a snapshot of the current device state.
    #[must_use]
    pub fn snapshot(&self) -> RemoteDeviceSnapshot {
        RemoteDeviceSnapshot::from_state(&self.state.read().expect("device state lock poisoned"))
    }

    /// Resets a channel to absolute zero intensity.
    pub async fn reset_intensity(
        &self,
        channel: Channel,
        options: OperationOptions,
    ) -> Result<OperationHandle, WsError> {
        self.submit_intensity(channel, 7, 0, None, None, options)
            .await
    }

    /// Adds intensity to a channel.
    pub async fn add_intensity(
        &self,
        channel: Channel,
        value: u16,
        options: OperationOptions,
    ) -> Result<OperationHandle, WsError> {
        let value = i32::from(value);
        if value > 200 {
            return Err(WsError::InvalidParams(
                "AddIntensity value exceeds 200".to_owned(),
            ));
        }
        self.submit_intensity(channel, 3, value, None, None, options)
            .await
    }

    /// Reduces intensity on a channel.
    pub async fn reduce_intensity(
        &self,
        channel: Channel,
        value: u16,
        options: OperationOptions,
    ) -> Result<OperationHandle, WsError> {
        let value = i32::from(value);
        if value > 200 {
            return Err(WsError::InvalidParams(
                "AddIntensity magnitude exceeds 200".to_owned(),
            ));
        }
        self.submit_intensity(channel, 3, -value, None, None, options)
            .await
    }

    /// Applies a temporary intensity for a duration.
    pub async fn set_temp_intensity(
        &self,
        channel: Channel,
        intensity: pulselink_core::Intensity,
        duration: std::time::Duration,
        options: OperationOptions,
    ) -> Result<OperationHandle, WsError> {
        self.submit_intensity(
            channel,
            4,
            i32::from(intensity.get()),
            Some(duration_to_ms(duration)?),
            None,
            options,
        )
        .await
    }

    /// Sends version-2 (three-byte) pulse frames to a channel.
    pub async fn send_pulse_v2(
        &self,
        channel: Channel,
        duration: std::time::Duration,
        frames: Vec<crate::operation::PulseFrameV2>,
        options: OperationOptions,
    ) -> Result<OperationHandle, WsError> {
        if frames.is_empty() {
            return Err(WsError::InvalidParams(
                "send_pulse_v2 requires at least one frame".to_owned(),
            ));
        }
        let frames = frames
            .into_iter()
            .map(|frame| {
                pulselink_protocol::socket::v4::PulseFrameWire::Numbers(frame.as_bytes().to_vec())
            })
            .collect();
        self.submit_value(
            channel,
            0,
            pulselink_protocol::socket::v4::OperateValue::PulseFrames(frames),
            Some(duration_to_ms(duration)?),
            Some(2),
            options,
        )
        .await
    }

    /// Sends version-3 pulse frames to a channel.
    pub async fn send_pulse(
        &self,
        channel: Channel,
        duration: std::time::Duration,
        frames: Vec<PulseFrame>,
        options: OperationOptions,
    ) -> Result<OperationHandle, WsError> {
        if frames.is_empty() {
            return Err(WsError::InvalidParams(
                "send_pulse requires at least one frame".to_owned(),
            ));
        }
        let frames = frames
            .into_iter()
            .map(|frame| {
                pulselink_protocol::socket::v4::PulseFrameWire::Numbers(frame.as_bytes().to_vec())
            })
            .collect();
        self.submit_value(
            channel,
            0,
            pulselink_protocol::socket::v4::OperateValue::PulseFrames(frames),
            Some(duration_to_ms(duration)?),
            Some(3),
            options,
        )
        .await
    }

    async fn submit_intensity(
        &self,
        channel: Channel,
        action: u8,
        value: i32,
        duration_ms: Option<u64>,
        ver: Option<u8>,
        options: OperationOptions,
    ) -> Result<OperationHandle, WsError> {
        self.submit_value(
            channel,
            action,
            pulselink_protocol::socket::v4::OperateValue::Intensity(value),
            duration_ms,
            ver,
            options,
        )
        .await
    }

    async fn submit_value(
        &self,
        channel: Channel,
        action: u8,
        value: pulselink_protocol::socket::v4::OperateValue,
        duration_ms: Option<u64>,
        ver: Option<u8>,
        options: OperationOptions,
    ) -> Result<OperationHandle, WsError> {
        let operate = pulselink_protocol::socket::v4::DeviceOperate {
            slot_id: self.slot_id.clone(),
            action: action_type(action)?,
            channel: channel.as_u8(),
            priority: options.priority,
            duration_ms,
            value,
            immediate: Some(options.immediate),
            ver,
            seq: None,
        };
        let pending = self.client.submit_operate(operate, options.timeout).await?;
        Ok(OperationHandle::new(pending))
    }
}

fn duration_to_ms(duration: std::time::Duration) -> Result<u64, WsError> {
    u64::try_from(duration.as_millis())
        .map_err(|_| WsError::InvalidParams("duration is too large".to_owned()))
}

fn action_type(value: u8) -> Result<pulselink_protocol::socket::v4::ActionType, WsError> {
    pulselink_protocol::socket::v4::ActionType::try_from(value)
        .map_err(|error| WsError::InvalidParams(error.to_string()))
}
