//! DG-LAB V4 app events.

use serde::{Deserialize, Serialize};

use crate::socket::v4::device::{RemoteDevice, SlotPatchEntry};

/// An event sent by an app client.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "ev")]
pub enum AppEvent {
    /// Full device list snapshot.
    #[serde(rename = "devices.snapshot")]
    DevicesSnapshot {
        /// Devices currently exposed by the app.
        devices: Vec<RemoteDevice>,
    },
    /// Incremental device list update.
    #[serde(rename = "devices.patch")]
    DevicesPatch {
        /// Added or replaced devices.
        #[serde(default)]
        added: Vec<RemoteDevice>,
        /// Removed slot ids.
        #[serde(default)]
        removed: Vec<String>,
    },
    /// Incremental slot state update.
    #[serde(rename = "slots.patch")]
    SlotsPatch {
        /// Changed slots.
        slots: Vec<SlotPatchEntry>,
    },
    /// A custom app action, expected in `0..=9`.
    #[serde(rename = "custom.action")]
    CustomAction {
        /// Action number.
        action: u8,
    },
    /// An event type unknown to this version of the protocol crate.
    #[serde(other)]
    Unknown,
}

impl AppEvent {
    /// Validates event-specific constraints that are not enforced by serde.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::ProtocolError::ValueOutOfRange`] when a custom
    /// action is outside `0..=9`.
    pub fn validate(&self) -> Result<(), crate::error::ProtocolError> {
        if let Self::CustomAction { action } = self
            && *action > 9
        {
            return Err(crate::error::ProtocolError::ValueOutOfRange {
                field: "custom action",
                value: i64::from(*action),
                min: 0,
                max: 9,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_snapshot() {
        let event: AppEvent = serde_json::from_str(
            r#"{"ev":"devices.snapshot","devices":[{"slotId":"slot-a","name":"A","type":"COYOTE_030"}]}"#,
        )
        .expect("valid snapshot");
        assert!(matches!(
            event,
            AppEvent::DevicesSnapshot { ref devices } if devices.len() == 1
        ));
    }

    #[test]
    fn parses_patches() {
        let event: AppEvent =
            serde_json::from_str(r#"{"ev":"devices.patch","added":[],"removed":["slot-a"]}"#)
                .expect("valid device patch");
        assert_eq!(
            event,
            AppEvent::DevicesPatch {
                added: Vec::new(),
                removed: vec!["slot-a".to_owned()]
            }
        );

        let event: AppEvent = serde_json::from_str(
            r#"{"ev":"slots.patch","slots":[{"slotId":"slot-a","props":{"strength":10}}]}"#,
        )
        .expect("valid slot patch");
        assert!(matches!(
            event,
            AppEvent::SlotsPatch { ref slots } if slots.len() == 1
        ));
    }

    #[test]
    fn parses_custom_action_and_validates() {
        let event: AppEvent =
            serde_json::from_str(r#"{"ev":"custom.action","action":7}"#).expect("valid action");
        assert_eq!(event, AppEvent::CustomAction { action: 7 });
        assert!(event.validate().is_ok());

        let invalid = AppEvent::CustomAction { action: 10 };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn unknown_event_is_forward_compatible() {
        let event: AppEvent =
            serde_json::from_str(r#"{"ev":"future.event","data":1}"#).expect("unknown event");
        assert_eq!(event, AppEvent::Unknown);
    }
}
