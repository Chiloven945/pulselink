//! DG-LAB V4 device operation payloads.

use serde::{Deserialize, Serialize};

use crate::error::ProtocolError;

/// Supported V4 device operation types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
#[repr(u8)]
pub enum ActionType {
    /// Append pulse data to a channel.
    AppendPulseData = 0,
    /// Adjust intensity by a relative amount.
    AddIntensity = 3,
    /// Apply a temporary intensity for a duration.
    SetTempIntensity = 4,
    /// Set absolute intensity; V4 accepts only zero.
    SetIntensity = 7,
}

impl TryFrom<u8> for ActionType {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::AppendPulseData),
            3 => Ok(Self::AddIntensity),
            4 => Ok(Self::SetTempIntensity),
            7 => Ok(Self::SetIntensity),
            other => Err(ProtocolError::UnsupportedActionType(other)),
        }
    }
}

impl From<ActionType> for u8 {
    fn from(value: ActionType) -> Self {
        value as Self
    }
}

/// The `v` field of a device operation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OperateValue {
    /// An intensity value.
    Intensity(i32),
    /// One or more pulse frames.
    PulseFrames(Vec<PulseFrameWire>),
}

/// A pulse frame in either hexadecimal-string or numeric form.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PulseFrameWire {
    /// Hexadecimal bytes, such as `0A0A0A0A00000000`.
    Hex(String),
    /// Raw bytes.
    Numbers(Vec<u8>),
}

impl PulseFrameWire {
    /// Validates the frame length and hex encoding.
    pub fn validate(&self, expected_bytes: usize) -> Result<(), ProtocolError> {
        match self {
            Self::Hex(hex) => {
                if hex.len() != expected_bytes * 2 {
                    return Err(ProtocolError::InvalidPulseFrame(
                        "hex frame has the wrong byte length",
                    ));
                }
                if !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                    return Err(ProtocolError::InvalidPulseFrame(
                        "hex frame contains a non-hex character",
                    ));
                }
                Ok(())
            }
            Self::Numbers(bytes) => {
                if bytes.len() != expected_bytes {
                    return Err(ProtocolError::InvalidPulseFrame(
                        "numeric frame has the wrong byte length",
                    ));
                }
                Ok(())
            }
        }
    }
}

/// A V4 `device.op` request payload.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeviceOperate {
    /// Device slot id.
    #[serde(rename = "s")]
    pub slot_id: String,
    /// Operation type.
    #[serde(rename = "t")]
    pub action: ActionType,
    /// Channel discriminator: `0` = A, `1` = B.
    #[serde(rename = "c")]
    pub channel: u8,
    /// Optional priority `0..=2`.
    #[serde(default, rename = "p")]
    pub priority: Option<u8>,
    /// Optional duration in milliseconds.
    #[serde(default, rename = "d")]
    pub duration_ms: Option<u64>,
    /// Operation value.
    #[serde(rename = "v")]
    pub value: OperateValue,
    /// Whether to replace existing tasks with the same device/channel/type.
    #[serde(default, rename = "im")]
    pub immediate: Option<bool>,
    /// Pulse frame version: `2` or `3`.
    #[serde(default)]
    pub ver: Option<u8>,
    /// Legacy frame sequence number.
    #[serde(default)]
    pub seq: Option<u32>,
}

impl DeviceOperate {
    /// Validates cross-field constraints that serde cannot express.
    pub fn validate(&self) -> Result<(), ProtocolError> {
        if self.slot_id.is_empty() {
            return Err(ProtocolError::InvalidParameters("slotId must not be empty"));
        }

        if !matches!(self.channel, 0 | 1) {
            return Err(ProtocolError::ValueOutOfRange {
                field: "device channel",
                value: i64::from(self.channel),
                min: 0,
                max: 1,
            });
        }

        if let Some(priority) = self.priority
            && priority > 2
        {
            return Err(ProtocolError::ValueOutOfRange {
                field: "device operation priority",
                value: i64::from(priority),
                min: 0,
                max: 2,
            });
        }

        match self.action {
            ActionType::AppendPulseData => self.validate_pulse_frames(),
            ActionType::AddIntensity => match self.value {
                OperateValue::Intensity(value) if (-200..=200).contains(&value) => Ok(()),
                OperateValue::Intensity(_) => Err(ProtocolError::ValueOutOfRange {
                    field: "AddIntensity value",
                    value: i64::from(self.intensity_value()),
                    min: -200,
                    max: 200,
                }),
                OperateValue::PulseFrames(_) => Err(ProtocolError::InvalidParameters(
                    "AddIntensity requires an integer value",
                )),
            },
            ActionType::SetTempIntensity => match self.value {
                OperateValue::Intensity(value) if (0..=200).contains(&value) => Ok(()),
                OperateValue::Intensity(_) => Err(ProtocolError::ValueOutOfRange {
                    field: "SetTempIntensity value",
                    value: i64::from(self.intensity_value()),
                    min: 0,
                    max: 200,
                }),
                OperateValue::PulseFrames(_) => Err(ProtocolError::InvalidParameters(
                    "SetTempIntensity requires an integer value",
                )),
            },
            ActionType::SetIntensity => match self.value {
                OperateValue::Intensity(0) => Ok(()),
                OperateValue::Intensity(_) => Err(ProtocolError::InvalidParameters(
                    "SetIntensity accepts only value 0",
                )),
                OperateValue::PulseFrames(_) => Err(ProtocolError::InvalidParameters(
                    "SetIntensity requires an integer value",
                )),
            },
        }
    }

    fn intensity_value(&self) -> i32 {
        match self.value {
            OperateValue::Intensity(value) => value,
            OperateValue::PulseFrames(_) => 0,
        }
    }

    fn validate_pulse_frames(&self) -> Result<(), ProtocolError> {
        let version = self.ver.unwrap_or(3);
        if !matches!(version, 2 | 3) {
            return Err(ProtocolError::ValueOutOfRange {
                field: "pulse frame version",
                value: i64::from(version),
                min: 2,
                max: 3,
            });
        }

        let expected_bytes = match version {
            2 => 3,
            3 => 8,
            _ => unreachable!("version was validated above"),
        };

        match &self.value {
            OperateValue::PulseFrames(frames) => {
                if frames.is_empty() {
                    return Err(ProtocolError::InvalidPulseFrame(
                        "AppendPulseData requires at least one frame",
                    ));
                }
                for frame in frames {
                    frame.validate(expected_bytes)?;
                }
                Ok(())
            }
            OperateValue::Intensity(_) => Err(ProtocolError::InvalidParameters(
                "AppendPulseData requires pulse frames",
            )),
        }
    }
}

/// A V4 `device.op.clear` request payload.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ClearOperate {
    /// Optional device slot id.
    #[serde(default, rename = "s")]
    pub slot_id: Option<String>,
    /// Optional channel discriminator; requires `slot_id`.
    #[serde(default, rename = "c")]
    pub channel: Option<u8>,
}

impl ClearOperate {
    /// Validates that a channel is not specified without a slot id.
    pub fn validate(&self) -> Result<(), ProtocolError> {
        if let Some(channel) = self.channel {
            if self.slot_id.is_none() {
                return Err(ProtocolError::InvalidParameters(
                    "device.op.clear channel requires slotId",
                ));
            }
            if !matches!(channel, 0 | 1) {
                return Err(ProtocolError::ValueOutOfRange {
                    field: "clear channel",
                    value: i64::from(channel),
                    min: 0,
                    max: 1,
                });
            }
        }

        Ok(())
    }
}

/// Why a device operation ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OperationEndReason {
    /// The task completed normally.
    Completed,
    /// The task was cleared.
    Cleared,
    /// The task was replaced.
    Replaced,
    /// The task was cancelled because of disconnection.
    Cancelled,
}

/// The result of a completed `device.op` request.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeviceOperateResult {
    /// Operation type that ended.
    #[serde(rename = "type")]
    pub action: ActionType,
    /// Why the task ended.
    pub reason: OperationEndReason,
    /// Device slot id, when provided.
    #[serde(default, rename = "slotId")]
    pub slot_id: Option<String>,
    /// Channel discriminator, when provided.
    #[serde(default)]
    pub channel: Option<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_operate(json: &str) -> DeviceOperate {
        serde_json::from_str(json).expect("valid device operation")
    }

    #[test]
    fn parses_all_action_types() {
        let append = parse_operate(r#"{"s":"slot-a","t":0,"c":0,"v":["0A0A0A0A00000000"]}"#);
        assert_eq!(append.action, ActionType::AppendPulseData);
        assert!(append.validate().is_ok());

        let add = parse_operate(r#"{"s":"slot-a","t":3,"c":0,"v":5}"#);
        assert_eq!(add.action, ActionType::AddIntensity);
        assert!(add.validate().is_ok());

        let temp = parse_operate(r#"{"s":"slot-a","t":4,"c":1,"d":3000,"v":30}"#);
        assert_eq!(temp.action, ActionType::SetTempIntensity);
        assert!(temp.validate().is_ok());

        let reset = parse_operate(r#"{"s":"slot-a","t":7,"c":1,"v":0}"#);
        assert_eq!(reset.action, ActionType::SetIntensity);
        assert!(reset.validate().is_ok());
    }

    #[test]
    fn set_intensity_rejects_non_zero() {
        let operate = parse_operate(r#"{"s":"slot-a","t":7,"c":0,"v":5}"#);
        assert!(matches!(
            operate.validate(),
            Err(ProtocolError::InvalidParameters(_))
        ));
    }

    #[test]
    fn add_intensity_accepts_negative_values() {
        let operate = parse_operate(r#"{"s":"slot-a","t":3,"c":0,"v":-7}"#);
        assert!(operate.validate().is_ok());
    }

    #[test]
    fn add_intensity_rejects_out_of_range() {
        let operate = parse_operate(r#"{"s":"slot-a","t":3,"c":0,"v":201}"#);
        assert!(matches!(
            operate.validate(),
            Err(ProtocolError::ValueOutOfRange { .. })
        ));
    }

    #[test]
    fn validates_frame_lengths() {
        let v3 = parse_operate(r#"{"s":"slot-a","t":0,"c":0,"v":["0A0A0A0A00000000"],"ver":3}"#);
        assert!(v3.validate().is_ok());

        let v2 = parse_operate(r#"{"s":"slot-a","t":0,"c":0,"v":[[1,2,3]],"ver":2}"#);
        assert!(v2.validate().is_ok());

        let bad = parse_operate(r#"{"s":"slot-a","t":0,"c":0,"v":["0A0A"],"ver":3}"#);
        assert!(matches!(
            bad.validate(),
            Err(ProtocolError::InvalidPulseFrame(_))
        ));
    }

    #[test]
    fn rejects_invalid_channel_and_priority() {
        let channel = parse_operate(r#"{"s":"slot-a","t":7,"c":2,"v":0}"#);
        assert!(matches!(
            channel.validate(),
            Err(ProtocolError::ValueOutOfRange { .. })
        ));

        let priority = parse_operate(r#"{"s":"slot-a","t":7,"c":0,"v":0,"p":3}"#);
        assert!(matches!(
            priority.validate(),
            Err(ProtocolError::ValueOutOfRange { .. })
        ));
    }

    #[test]
    fn rejects_unsupported_action_type() {
        assert!(
            serde_json::from_str::<DeviceOperate>(r#"{"s":"slot-a","t":5,"c":0,"v":0}"#).is_err()
        );
    }

    #[test]
    fn clear_validation() {
        let all: ClearOperate = serde_json::from_str("{}").expect("valid clear");
        assert!(all.validate().is_ok());

        let channel_only: ClearOperate =
            serde_json::from_str(r#"{"c":0}"#).expect("valid clear shape");
        assert!(channel_only.validate().is_err());

        let channel: ClearOperate =
            serde_json::from_str(r#"{"s":"slot-a","c":1}"#).expect("valid clear");
        assert!(channel.validate().is_ok());
    }

    #[test]
    fn parses_operation_result() {
        let result: DeviceOperateResult = serde_json::from_str(
            r#"{"type":7,"reason":"completed","slotId":"slot-a","channel":1}"#,
        )
        .expect("valid result");
        assert_eq!(result.action, ActionType::SetIntensity);
        assert_eq!(result.reason, OperationEndReason::Completed);
        assert_eq!(result.channel, Some(1));
    }
}
