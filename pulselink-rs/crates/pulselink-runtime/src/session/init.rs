//! Session initialization and BF settings.

use pulselink_core::{BatteryState, ChannelMap, ConnectionState, DeviceEvent, Observed};
use pulselink_protocol::coyote::v3::gatt::{CHAR_BATTERY, CHAR_NOTIFY, CHAR_WRITE};
use pulselink_protocol::coyote::v3::{BfCommand, decode_battery};
use pulselink_transport::{GattLink, WriteMode};
use tracing::debug;

use crate::error::RuntimeError;
use crate::event::CoyoteV3Event;
use crate::state::CoyoteV3Settings;

use super::actor::Actor;
use super::notification::spawn_notification_pump;

impl Actor {
    pub(super) async fn initialize(&mut self, link: Box<dyn GattLink>) -> Result<(), RuntimeError> {
        self.stop_notification_task();
        let notify_stream = link
            .subscribe(CHAR_NOTIFY)
            .await
            .map_err(RuntimeError::from)?;
        let battery_stream = link.subscribe(CHAR_BATTERY).await.ok();
        let battery = match link.read(CHAR_BATTERY).await {
            Ok(bytes) => decode_battery(&bytes).ok(),
            Err(error) => {
                debug!(%error, "battery read failed; continuing without battery");
                None
            }
        };

        let settings = self.settings.unwrap_or_else(|| self.default_settings());
        let bf = BfCommand {
            soft_limit: settings.soft_limit,
            frequency_balance: settings.frequency_balance,
            intensity_balance: settings.intensity_balance,
        };
        link.write(CHAR_WRITE, &bf.encode(), WriteMode::Auto)
            .await
            .map_err(RuntimeError::from)?;

        self.link = Some(link);
        self.settings = Some(settings);
        self.state.settings = Observed::Known(settings);
        self.state.common.connection = ConnectionState::Ready;
        self.state.common.battery = battery.map_or(Observed::Unknown, |level| {
            Observed::Known(BatteryState::new(level))
        });
        self.state.common.channels = ChannelMap::new(
            pulselink_core::ChannelState {
                intensity: Observed::Unknown,
                soft_limit: Observed::Known(settings.soft_limit.a),
            },
            pulselink_core::ChannelState {
                intensity: Observed::Unknown,
                soft_limit: Observed::Known(settings.soft_limit.b),
            },
        );
        self.state.pattern = None;
        self.state.last_b1 = None;
        self.sequence.reset();
        self.pending = None;
        self.queue.clear();
        self.playground = None;
        self.desired = ChannelMap::new(Observed::Unknown, Observed::Unknown);
        self.ticker_active = false;
        self.next_deadline = None;
        self.publish();
        self.emit(CoyoteV3Event::Device(DeviceEvent::ConnectionChanged {
            state: ConnectionState::Ready,
        }));

        self.notification_task = Some(spawn_notification_pump(
            notify_stream,
            battery_stream,
            self.notification_tx.clone(),
            self.cancel.clone(),
        ));
        Ok(())
    }

    pub(super) fn default_settings(&self) -> CoyoteV3Settings {
        let balance = self.config.balance.unwrap_or_default();
        CoyoteV3Settings {
            soft_limit: self.config.initial_soft_limit,
            frequency_balance: balance.frequency,
            intensity_balance: balance.intensity,
        }
    }

    pub(super) async fn write_settings(
        &mut self,
        settings: CoyoteV3Settings,
    ) -> Result<(), RuntimeError> {
        let Some(link) = &self.link else {
            return Err(RuntimeError::NotReady);
        };
        let bf = BfCommand {
            soft_limit: settings.soft_limit,
            frequency_balance: settings.frequency_balance,
            intensity_balance: settings.intensity_balance,
        };
        link.write(CHAR_WRITE, &bf.encode(), WriteMode::Auto)
            .await
            .map_err(RuntimeError::from)?;
        self.settings = Some(settings);
        Ok(())
    }
}
