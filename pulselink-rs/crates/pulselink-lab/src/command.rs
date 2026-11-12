//! Command-line parser for the lab.

use std::net::SocketAddr;

use pulselink::Channel;
use pulselink::DeviceKind;

/// A command executed by the background worker.
#[derive(Clone, Debug)]
pub enum Command {
    /// Scan all devices or one profile.
    Scan(Option<DeviceKind>),
    /// Connect a scan result by index.
    Connect(usize),
    /// Disconnect the active session.
    Disconnect,
    /// Set a Coyote V3 intensity.
    V3Intensity(Channel, u8),
    /// Set a Coyote V3 intensity temporarily.
    V3Temp(Channel, u8, u64),
    /// Play a built-in low-intensity demo pattern.
    V3PlayDemo,
    /// Stop Coyote V3 playback and output.
    V3Stop,
    /// Reconnect the Coyote V3 session.
    V3Reconnect,
    /// Set a Coyote V2 wire intensity.
    V2Intensity(Channel, u16),
    /// Set a Coyote V2 pulse parameter set.
    V2Pulse(Channel, u8, u16, u8),
    /// Set an Opossum intensity.
    OpossumIntensity(Channel, u8),
    /// Send an Opossum vibration frame.
    OpossumPulse(Channel, [u8; 4]),
    /// Start Civet pressure reporting.
    CivetStart,
    /// Stop Civet pressure reporting.
    CivetStop,
    /// Reset Civet pressure.
    CivetReset,
    /// Set Civet display orientation.
    CivetOrientation(u8),
    /// Configure `PawPrints` no-trigger mode.
    PawNoTrigger,
    /// Reset the active `PawPrints` parameter.
    PawReset,
    /// Start `PawPrints` automatic angle detection.
    PawAngles,
    /// Set the `PawPrints` shoulder-light color.
    PawLight(u8),
    /// Connect a V4 relay URL.
    WsConnect(String),
    /// Start a local real V4 relay and connect this controller.
    WsLocal(Option<SocketAddr>),
    /// Start a local real V4 relay without connecting a controller.
    RelayStart(Option<SocketAddr>),
    /// Stop the local V4 relay.
    RelayStop,
    /// Report local relay status.
    RelayStatus,
    /// Refresh the V4 device list.
    WsRefresh,
    /// Set a V4 device intensity.
    WsIntensity {
        /// Device slot id.
        slot: String,
        /// Target channel.
        channel: Channel,
        /// Requested intensity.
        value: u8,
    },
    /// Reset a V4 device channel.
    WsReset {
        /// Device slot id.
        slot: String,
        /// Target channel.
        channel: Channel,
    },
    /// Clear all V4 operations.
    WsClear,
    /// Enable or disable real output.
    SetOutput(bool),
    /// Set the lab maximum intensity.
    SetMaxIntensity(u8),
    /// Emergency-stop all active local and V4 output.
    StopAll,
    /// Stop the worker.
    Shutdown,
}

/// A parsed input action.
#[derive(Clone, Debug)]
pub enum InputCommand {
    /// Send a command to the worker.
    Worker(Command),
    /// Show the help tab.
    Help,
    /// Quit the lab.
    Quit,
}

/// Parses one command line.
pub fn parse(input: &str) -> Result<InputCommand, String> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let Some(head) = parts.first().copied() else {
        return Err("empty command".to_owned());
    };
    match head {
        "help" => Ok(InputCommand::Help),
        "quit" | "q" => Ok(InputCommand::Quit),
        "scan" => Ok(InputCommand::Worker(Command::Scan(
            parts.get(1).map(|value| parse_kind(value)).transpose()?,
        ))),
        "connect" => Ok(InputCommand::Worker(Command::Connect(parse_usize(
            parts.get(1).ok_or("connect requires an index")?,
        )?))),
        "disconnect" => Ok(InputCommand::Worker(Command::Disconnect)),
        "int" => {
            let channel = parse_channel(parts.get(1).ok_or("int requires a channel")?)?;
            let value = parse_u8(parts.get(2).ok_or("int requires a value")?)?;
            Ok(InputCommand::Worker(Command::V3Intensity(channel, value)))
        }
        "temp" => {
            let channel = parse_channel(parts.get(1).ok_or("temp requires a channel")?)?;
            let value = parse_u8(parts.get(2).ok_or("temp requires a value")?)?;
            let duration = parse_u64(parts.get(3).ok_or("temp requires a duration")?)?;
            Ok(InputCommand::Worker(Command::V3Temp(
                channel, value, duration,
            )))
        }
        "play" => Ok(InputCommand::Worker(Command::V3PlayDemo)),
        "stop" => Ok(InputCommand::Worker(Command::V3Stop)),
        "reconnect" => Ok(InputCommand::Worker(Command::V3Reconnect)),
        "v2" => parse_v2(&parts[1..]),
        "op" => parse_opossum(&parts[1..]),
        "civet" => parse_civet(&parts[1..]),
        "paw" => parse_paw(&parts[1..]),
        "ws" => parse_ws(&parts[1..]),
        "relay" => parse_relay(&parts[1..]),
        "output" => match parts.get(1).copied() {
            Some("on") => Ok(InputCommand::Worker(Command::SetOutput(true))),
            Some("off") => Ok(InputCommand::Worker(Command::SetOutput(false))),
            _ => Err("usage: output on|off".to_owned()),
        },
        "max" => Ok(InputCommand::Worker(Command::SetMaxIntensity(parse_u8(
            parts.get(1).ok_or("max requires a value")?,
        )?))),
        "stopall" => Ok(InputCommand::Worker(Command::StopAll)),
        other => Err(format!("unknown command: {other}")),
    }
}

fn parse_v2(parts: &[&str]) -> Result<InputCommand, String> {
    match parts.first().copied() {
        Some("int") => {
            let channel = parse_channel(parts.get(1).ok_or("v2 int requires a channel")?)?;
            let value = parse_u16(parts.get(2).ok_or("v2 int requires a value")?)?;
            Ok(InputCommand::Worker(Command::V2Intensity(channel, value)))
        }
        Some("pulse") => {
            let channel = parse_channel(parts.get(1).ok_or("v2 pulse requires a channel")?)?;
            let x = parse_u8(parts.get(2).ok_or("v2 pulse requires x")?)?;
            let y = parse_u16(parts.get(3).ok_or("v2 pulse requires y")?)?;
            let z = parse_u8(parts.get(4).ok_or("v2 pulse requires z")?)?;
            Ok(InputCommand::Worker(Command::V2Pulse(channel, x, y, z)))
        }
        _ => Err("usage: v2 int|pulse ...".to_owned()),
    }
}

fn parse_opossum(parts: &[&str]) -> Result<InputCommand, String> {
    match parts.first().copied() {
        Some("int") => {
            let channel = parse_channel(parts.get(1).ok_or("op int requires a channel")?)?;
            let value = parse_u8(parts.get(2).ok_or("op int requires a value")?)?;
            Ok(InputCommand::Worker(Command::OpossumIntensity(
                channel, value,
            )))
        }
        Some("pulse") => {
            let channel = parse_channel(parts.get(1).ok_or("op pulse requires a channel")?)?;
            let values = [
                parse_u8(parts.get(2).ok_or("op pulse requires 4 values")?)?,
                parse_u8(parts.get(3).ok_or("op pulse requires 4 values")?)?,
                parse_u8(parts.get(4).ok_or("op pulse requires 4 values")?)?,
                parse_u8(parts.get(5).ok_or("op pulse requires 4 values")?)?,
            ];
            Ok(InputCommand::Worker(Command::OpossumPulse(channel, values)))
        }
        _ => Err("usage: op int|pulse ...".to_owned()),
    }
}

fn parse_civet(parts: &[&str]) -> Result<InputCommand, String> {
    match parts.first().copied() {
        Some("start") => Ok(InputCommand::Worker(Command::CivetStart)),
        Some("stop") => Ok(InputCommand::Worker(Command::CivetStop)),
        Some("reset") => Ok(InputCommand::Worker(Command::CivetReset)),
        Some("orientation") => Ok(InputCommand::Worker(Command::CivetOrientation(parse_u8(
            parts.get(1).ok_or("civet orientation requires a value")?,
        )?))),
        _ => Err("usage: civet start|stop|reset|orientation N".to_owned()),
    }
}

fn parse_paw(parts: &[&str]) -> Result<InputCommand, String> {
    match parts.first().copied() {
        Some("notrigger") => Ok(InputCommand::Worker(Command::PawNoTrigger)),
        Some("reset") => Ok(InputCommand::Worker(Command::PawReset)),
        Some("angles") => Ok(InputCommand::Worker(Command::PawAngles)),
        Some("light") => Ok(InputCommand::Worker(Command::PawLight(parse_u8(
            parts.get(1).ok_or("paw light requires a color")?,
        )?))),
        _ => Err("usage: paw notrigger|reset|angles|light N".to_owned()),
    }
}

fn parse_relay(parts: &[&str]) -> Result<InputCommand, String> {
    match parts.first().copied() {
        Some("start") => {
            let bind = parts
                .get(1)
                .map(|value| {
                    value
                        .parse::<SocketAddr>()
                        .map_err(|_| format!("invalid bind: {value}"))
                })
                .transpose()?;
            Ok(InputCommand::Worker(Command::RelayStart(bind)))
        }
        Some("stop") => Ok(InputCommand::Worker(Command::RelayStop)),
        Some("status") => Ok(InputCommand::Worker(Command::RelayStatus)),
        _ => Err("usage: relay start [addr]|stop|status".to_owned()),
    }
}

fn parse_ws(parts: &[&str]) -> Result<InputCommand, String> {
    match parts.first().copied() {
        Some("connect") => Ok(InputCommand::Worker(Command::WsConnect(
            parts.get(1).ok_or("ws connect requires a URL")?.to_string(),
        ))),
        Some("local") => {
            let bind = parts
                .get(1)
                .map(|value| {
                    value
                        .parse::<SocketAddr>()
                        .map_err(|_| format!("invalid bind: {value}"))
                })
                .transpose()?;
            Ok(InputCommand::Worker(Command::WsLocal(bind)))
        }
        Some("refresh") => Ok(InputCommand::Worker(Command::WsRefresh)),
        Some("int") => {
            let slot = parts.get(1).ok_or("ws int requires a slot")?.to_string();
            let channel = parse_channel(parts.get(2).ok_or("ws int requires a channel")?)?;
            let value = parse_u8(parts.get(3).ok_or("ws int requires a value")?)?;
            Ok(InputCommand::Worker(Command::WsIntensity {
                slot,
                channel,
                value,
            }))
        }
        Some("reset") => {
            let slot = parts.get(1).ok_or("ws reset requires a slot")?.to_string();
            let channel = parse_channel(parts.get(2).ok_or("ws reset requires a channel")?)?;
            Ok(InputCommand::Worker(Command::WsReset { slot, channel }))
        }
        Some("clear") => Ok(InputCommand::Worker(Command::WsClear)),
        _ => Err("usage: ws connect|local|refresh|int|reset|clear".to_owned()),
    }
}

fn parse_kind(value: &str) -> Result<DeviceKind, String> {
    match value {
        "v2" | "coyote-v2" => Ok(DeviceKind::CoyoteV2),
        "v3" | "coyote-v3" => Ok(DeviceKind::CoyoteV3),
        "paw" | "pawprints" => Ok(DeviceKind::PawPrints),
        "civet" => Ok(DeviceKind::Civet),
        "opossum" | "op" => Ok(DeviceKind::Opossum),
        other => Err(format!("unknown device kind: {other}")),
    }
}

fn parse_channel(value: &str) -> Result<Channel, String> {
    match value {
        "a" | "A" => Ok(Channel::A),
        "b" | "B" => Ok(Channel::B),
        other => Err(format!("unknown channel: {other}")),
    }
}

fn parse_u8(value: &str) -> Result<u8, String> {
    value.parse().map_err(|_| format!("invalid u8: {value}"))
}

fn parse_u16(value: &str) -> Result<u16, String> {
    value.parse().map_err(|_| format!("invalid u16: {value}"))
}

fn parse_u64(value: &str) -> Result<u64, String> {
    value.parse().map_err(|_| format!("invalid u64: {value}"))
}

fn parse_usize(value: &str) -> Result<usize, String> {
    value.parse().map_err(|_| format!("invalid index: {value}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_device_commands() {
        assert!(matches!(
            parse("int a 10").expect("parse"),
            InputCommand::Worker(Command::V3Intensity(Channel::A, 10))
        ));
        assert!(matches!(
            parse("v2 pulse b 1 9 20").expect("parse"),
            InputCommand::Worker(Command::V2Pulse(Channel::B, 1, 9, 20))
        ));
        assert!(matches!(
            parse("op pulse a 1 2 3 4").expect("parse"),
            InputCommand::Worker(Command::OpossumPulse(Channel::A, [1, 2, 3, 4]))
        ));
    }

    #[test]
    fn parses_relay_commands() {
        assert!(matches!(
            parse("relay start 127.0.0.1:0").expect("parse"),
            InputCommand::Worker(Command::RelayStart(Some(_)))
        ));
        assert!(matches!(
            parse("relay start").expect("parse"),
            InputCommand::Worker(Command::RelayStart(None))
        ));
        assert!(matches!(
            parse("relay stop").expect("parse"),
            InputCommand::Worker(Command::RelayStop)
        ));
        assert!(parse("relay start not-an-addr").is_err());
    }

    #[test]
    fn rejects_invalid_commands() {
        assert!(parse("").is_err());
        assert!(parse("int c 10").is_err());
        assert!(parse("connect").is_err());
        assert!(parse("ws int slot c 10").is_err());
    }
}
