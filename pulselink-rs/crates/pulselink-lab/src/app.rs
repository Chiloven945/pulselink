//! Terminal application loop and key handling.

use std::io::{self, stdout};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::command::{self, Command, InputCommand};
use crate::model::{LabState, Update};
use crate::ui::{self, Tab};
use crate::worker::LabWorker;

/// The terminal lab application.
pub struct LabApp {
    worker: LabWorker,
    state: LabState,
    tab: Tab,
    input: Option<String>,
    status: String,
    quit: bool,
}

impl LabApp {
    /// Creates a lab app around a worker.
    #[must_use]
    pub fn new(worker: LabWorker) -> Self {
        Self {
            worker,
            state: LabState::default(),
            tab: Tab::Devices,
            input: None,
            status: "s=scan  :=command  F1/?=help  F12=stop".to_owned(),
            quit: false,
        }
    }

    /// Runs the terminal event loop.
    pub fn run(mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut out = stdout();
        execute!(out, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(out);
        let mut terminal = Terminal::new(backend)?;

        let result = self.event_loop(&mut terminal);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        self.worker.send(Command::StopAll);
        std::thread::sleep(Duration::from_millis(200));
        self.worker.shutdown();
        result
    }

    fn event_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        loop {
            self.apply_updates();
            terminal.draw(|frame| {
                ui::render(
                    frame,
                    &self.state,
                    self.tab,
                    self.input.as_deref(),
                    &self.status,
                );
            })?;

            if event::poll(Duration::from_millis(100))? {
                let event = event::read()?;
                if let Event::Key(key) = event
                    && key.kind == KeyEventKind::Press
                {
                    self.handle_key(key);
                }
            }
            if self.quit {
                break;
            }
        }
        Ok(())
    }

    fn apply_updates(&mut self) {
        while let Some(update) = self.worker.try_recv() {
            if let Update::Error(error) = &update {
                self.status.clone_from(error);
            }
            self.state.apply(update);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if self.input.is_some() {
            self.handle_input_key(key);
            return;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.quit = true;
            return;
        }

        match key.code {
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char(':') => self.input = Some(String::new()),
            KeyCode::Tab => self.tab = self.tab.next(),
            KeyCode::BackTab => self.tab = self.tab.previous(),
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.selected = self.state.selected.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.state.devices.is_empty() {
                    self.state.selected =
                        (self.state.selected + 1).min(self.state.devices.len() - 1);
                }
            }
            KeyCode::Enter => {
                if self.state.selected_device().is_some() {
                    self.worker.send(Command::Connect(self.state.selected));
                    self.status = format!("connecting device {}", self.state.selected);
                } else {
                    "no device selected; press s to scan".clone_into(&mut self.status);
                }
            }
            KeyCode::Char('s') => {
                self.worker.send(Command::Scan(None));
                "scanning all profiles".clone_into(&mut self.status);
            }
            KeyCode::Char('d') => {
                self.worker.send(Command::Disconnect);
                "disconnecting".clone_into(&mut self.status);
            }
            KeyCode::Char(' ') => {
                self.state.output_enabled = !self.state.output_enabled;
                self.worker
                    .send(Command::SetOutput(self.state.output_enabled));
                self.status = format!(
                    "output {}",
                    if self.state.output_enabled {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );
            }
            KeyCode::F(12) => {
                self.worker.send(Command::StopAll);
                "emergency stop".clone_into(&mut self.status);
            }
            KeyCode::F(1) | KeyCode::Char('?') => self.tab = Tab::Help,
            _ => {}
        }
    }

    fn handle_input_key(&mut self, key: KeyEvent) {
        let Some(input) = &mut self.input else {
            return;
        };
        match key.code {
            KeyCode::Esc => {
                self.input = None;
                "command cancelled".clone_into(&mut self.status);
            }
            KeyCode::Enter => {
                let value = std::mem::take(input);
                self.input = None;
                self.execute_input(&value);
            }
            KeyCode::Backspace => {
                input.pop();
            }
            KeyCode::Char(c) => input.push(c),
            KeyCode::Tab => input.push(' '),
            _ => {}
        }
    }

    fn execute_input(&mut self, input: &str) {
        match command::parse(input) {
            Ok(InputCommand::Worker(command)) => {
                self.status = format!("command: {input}");
                self.worker.send(command);
            }
            Ok(InputCommand::Help) => {
                self.tab = Tab::Help;
                "help".clone_into(&mut self.status);
            }
            Ok(InputCommand::Quit) => self.quit = true,
            Err(error) => self.status = error,
        }
    }
}
