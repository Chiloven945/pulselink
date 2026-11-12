//! Terminal rendering for the lab.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

use crate::model::{LabState, LogLevel};

/// Visible lab tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    /// Device discovery and connection.
    Devices,
    /// Active session controls.
    Session,
    /// V4 WebSocket controller.
    V4,
    /// Event log.
    Log,
    /// Key and command help.
    Help,
}

impl Tab {
    /// Next tab.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Devices => Self::Session,
            Self::Session => Self::V4,
            Self::V4 => Self::Log,
            Self::Log => Self::Help,
            Self::Help => Self::Devices,
        }
    }

    /// Previous tab.
    #[must_use]
    pub const fn previous(self) -> Self {
        match self {
            Self::Devices => Self::Help,
            Self::Session => Self::Devices,
            Self::V4 => Self::Session,
            Self::Log => Self::V4,
            Self::Help => Self::Log,
        }
    }

    /// Display name.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Devices => "Devices",
            Self::Session => "Session",
            Self::V4 => "V4",
            Self::Log => "Log",
            Self::Help => "Help",
        }
    }
}

/// Renders the entire lab.
pub fn render(
    frame: &mut Frame<'_>,
    state: &LabState,
    tab: Tab,
    input: Option<&str>,
    status: &str,
) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);

    render_header(frame, chunks[0], state, tab);
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(42), Constraint::Min(30)])
        .split(chunks[1]);
    render_devices(frame, body[0], state);
    render_tab(frame, body[1], state, tab);
    render_footer(frame, chunks[2], state, input, status);
}

fn render_header(frame: &mut Frame<'_>, area: Rect, state: &LabState, tab: Tab) {
    let output = if state.output_enabled {
        Span::styled(
            "OUTPUT ON",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled("OUTPUT OFF", Style::default().fg(Color::Green))
    };
    let connection = state
        .connected
        .as_ref()
        .map_or_else(|| "disconnected".to_owned(), |kind| format!("{kind:?}"));
    let line = Line::from(vec![
        Span::styled(
            "pulselink lab",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        output,
        Span::raw(format!("  session: {connection}  ")),
        Span::styled(
            format!("[{}]", tab.title()),
            Style::default().fg(Color::Cyan),
        ),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn render_devices(frame: &mut Frame<'_>, area: Rect, state: &LabState) {
    let items: Vec<ListItem<'_>> = state
        .devices
        .iter()
        .enumerate()
        .map(|(index, device)| {
            let marker = if index == state.selected { ">" } else { " " };
            let name = device.name.as_deref().unwrap_or("(unnamed)");
            let rssi = device
                .rssi
                .map_or_else(|| "-".to_owned(), |value| value.to_string());
            let text = format!("{marker} {index:>2} {name:<16} {rssi:>4} dBm");
            let style = if index == state.selected {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default()
            };
            ListItem::new(text).style(style)
        })
        .collect();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" devices: s=scan Enter=connect d=disconnect ");
    frame.render_widget(List::new(items).block(block), area);
}

fn render_tab(frame: &mut Frame<'_>, area: Rect, state: &LabState, tab: Tab) {
    let text = match tab {
        Tab::Devices => devices_text(state),
        Tab::Session => session_text(state),
        Tab::V4 => v4_text(state),
        Tab::Log => log_text(state),
        Tab::Help => help_text(),
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", tab.title()));
    frame.render_widget(
        Paragraph::new(text).block(block).wrap(Wrap { trim: false }),
        area,
    );
}

fn devices_text(state: &LabState) -> Text<'static> {
    let Some(device) = state.selected_device() else {
        return Text::from("No devices. Press s to scan, or :scan v3.");
    };
    Text::from(vec![
        Line::from(format!(
            "name: {}",
            device.name.as_deref().unwrap_or("(unnamed)")
        )),
        Line::from(format!("id: {}", device.id)),
        Line::from(format!("rssi: {:?} dBm", device.rssi)),
        Line::from(format!("services: {:?}", device.service_uuids)),
        Line::from(""),
        Line::from("Enter: connect selected"),
        Line::from(""),
        Line::from("Unnamed devices with 0x180C may be a Coyote V3 in privacy broadcast."),
        Line::from("Switch to public broadcast if connection fails."),
        Line::from(":scan [v2|v3|paw|civet|opossum]"),
        Line::from(":connect <index>"),
    ])
}

fn session_text(state: &LabState) -> Text<'static> {
    let Some(kind) = state.connected.clone() else {
        return Text::from("No active session.");
    };
    let mut lines = vec![
        Line::from(format!("kind: {kind:?}")),
        Line::from(format!(
            "connection: {:?}",
            state.common.as_ref().map(|common| common.connection)
        )),
        Line::from(format!(
            "battery: {:?}",
            state
                .common
                .as_ref()
                .and_then(|common| common.battery.copied())
        )),
    ];
    if let Some(v3) = &state.v3 {
        lines.push(Line::from(format!("V3 pattern: {:?}", v3.pattern)));
        lines.push(Line::from(format!("V3 last B1: {:?}", v3.last_b1)));
    }
    if let Some(pressure) = state.last_pressure() {
        lines.push(Line::from(format!("pressure: {pressure:.2} kPa")));
    }
    if !state.pressure.is_empty() {
        lines.push(Line::from(format!(
            "pressure history: {}",
            sparkline(&state.pressure)
        )));
    }
    lines.push(Line::from(format!("buttons: {:?}", state.buttons)));
    lines.push(Line::from(""));
    lines.push(Line::from("commands:"));
    lines.push(Line::from(
        "  int a 10        temp a 10 500      play      stop",
    ));
    lines.push(Line::from("  v2 int a 70     v2 pulse a 1 9 20"));
    lines.push(Line::from("  op int a 10     op pulse a 20 20 20 20"));
    lines.push(Line::from("  civet start|stop|reset|orientation 1"));
    lines.push(Line::from("  paw notrigger|reset|angles|light 1"));
    Text::from(lines)
}

fn v4_text(state: &LabState) -> Text<'static> {
    let mut lines = vec![
        Line::from(format!(
            "target id: {}",
            state.ws_target.as_deref().unwrap_or("not connected")
        )),
        Line::from(format!("devices: {}", state.ws_devices.len())),
        Line::from(""),
    ];
    for device in &state.ws_devices {
        lines.push(Line::from(format!(
            "{}  {}  {}  {:?}",
            device.slot_id,
            device.name,
            device.device_type,
            device.typed()
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from("commands:"));
    lines.push(Line::from("  ws connect wss://..."));
    lines.push(Line::from("  ws local"));
    lines.push(Line::from("  relay start [addr]"));
    lines.push(Line::from("  relay stop|status"));
    lines.push(Line::from("  ws refresh"));
    lines.push(Line::from("  ws int <slot> a 10"));
    lines.push(Line::from("  ws reset <slot> a"));
    lines.push(Line::from("  ws clear"));
    Text::from(lines)
}

fn log_text(state: &LabState) -> Text<'static> {
    let lines: Vec<Line<'_>> = state
        .log
        .iter()
        .rev()
        .take(20)
        .map(|entry| {
            let color = match entry.level {
                LogLevel::Info => Color::Gray,
                LogLevel::Warn => Color::Yellow,
                LogLevel::Error => Color::Red,
            };
            Line::from(Span::styled(
                format!(
                    "{:>7.1}s {:?} {}",
                    entry.at.elapsed().as_secs_f32(),
                    entry.level,
                    entry.message
                ),
                Style::default().fg(color),
            ))
        })
        .collect();
    Text::from(lines)
}

fn help_text() -> Text<'static> {
    Text::from(vec![
        Line::from("Keys"),
        Line::from("  :       command mode      Enter  connect selected"),
        Line::from("  s       scan             d      disconnect"),
        Line::from("  Tab     next tab          Shift+Tab previous tab"),
        Line::from("  Space   output gate       F12    emergency stop"),
        Line::from("  q       quit              F1/?   help"),
        Line::from(""),
        Line::from("Commands"),
        Line::from("  scan [v2|v3|paw|civet|opossum]   connect <index>"),
        Line::from("  output on|off   max <0..200>     stopall"),
        Line::from("  int|temp|play|stop|reconnect"),
        Line::from("  v2 int|pulse   op int|pulse   civet ...   paw ..."),
        Line::from("  ws connect|local|refresh|int|reset|clear"),
        Line::from("  relay start|stop|status"),
        Line::from(""),
        Line::from("Output is disabled by default. Emergency stop always attempts zero."),
    ])
}

fn render_footer(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &LabState,
    input: Option<&str>,
    status: &str,
) {
    let line = if let Some(input) = input {
        Line::from(vec![
            Span::styled(":", Style::default().fg(Color::Cyan)),
            Span::raw(input),
            Span::styled("_", Style::default().fg(Color::Cyan)),
        ])
    } else {
        Line::from(vec![
            Span::styled("status: ", Style::default().fg(Color::DarkGray)),
            Span::raw(status),
            Span::raw(format!(
                "   output={} max={} devices={} log={}",
                state.output_enabled,
                state.max_intensity,
                state.devices.len(),
                state.log.len()
            )),
        ])
    };
    let block = Block::default().borders(Borders::ALL).title(" input ");
    frame.render_widget(Paragraph::new(line).block(block), area);
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn sparkline(values: &[f32]) -> String {
    const BARS: [char; 8] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '█'];
    let max = values.iter().copied().fold(0.0_f32, f32::max).max(0.001);
    values
        .iter()
        .map(|value| {
            let index = ((value / max) * 7.0).round().clamp(0.0, 7.0) as usize;
            BARS[index]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn renders_without_panicking() {
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let state = LabState::default();
        terminal
            .draw(|frame| render(frame, &state, Tab::Help, None, "test"))
            .expect("draw");
    }

    #[test]
    fn tabs_cycle() {
        assert_eq!(Tab::Devices.next(), Tab::Session);
        assert_eq!(Tab::Devices.previous(), Tab::Help);
    }
}
