use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::Frame;
use ratatui_explorer::FileExplorer;

use crate::{
    config::DecklistConfig,
    startup::StartupChecks,
    tui::core::{ui, MenuTabs, Tui},
};

#[derive(Debug, Default)]
pub enum SupportedOS {
    #[default]
    Linux,
    Windows,
    Mac,
    Unsupported,
}

pub struct App {
    exit: bool,
    pub startup: bool,
    pub os: SupportedOS,
    pub directory_exist: bool,
    pub config_exist: bool,
    pub database_exist: bool,
    pub collection_exist: bool,
    pub directory_status: String,
    pub config_status: String,
    pub database_status: String,
    pub collection_status: String,
    pub config: DecklistConfig,
    pub active_tab: MenuTabs,
    pub startup_channel: (
        std::sync::mpsc::Sender<StartupChecks>,
        std::sync::mpsc::Receiver<StartupChecks>,
    ),
}

impl Default for App {
    fn default() -> Self {
        Self {
            exit: false,
            startup: false,
            os: SupportedOS::default(),
            directory_exist: false,
            config_exist: false,
            database_exist: false,
            collection_exist: false,
            directory_status: "Waiting on startup checks...".to_string(),
            config_status: "Waiting on startup checks...".to_string(),
            database_status: "Waiting on startup checks...".to_string(),
            collection_status: "Waiting on startup checks...".to_string(),
            config: DecklistConfig::default(),
            active_tab: MenuTabs::default(),
            startup_channel: std::sync::mpsc::channel(),
        }
    }
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(
        &mut self,
        terminal: &mut Tui,
        explorer: &mut FileExplorer,
        explorer2: &mut FileExplorer,
    ) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame, explorer, explorer2))?;
            self.handle_events(explorer, explorer2)?;
        }
        Ok(())
    }

    /// render the frame
    fn render_frame(
        &mut self,
        frame: &mut Frame,
        explorer: &mut FileExplorer,
        explorer2: &mut FileExplorer,
    ) {
        ui(frame, self, explorer, explorer2);
    }

    /// updates application state based on user input
    fn handle_events(
        &mut self,
        explorer: &mut FileExplorer,
        explorer2: &mut FileExplorer,
    ) -> io::Result<()> {
        let event = event::read()?;
        match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _ => {}
        };
        // TODO: handle file explorer tabs inputs here
        Ok(())
    }

    /// key events
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            // TODO: delete after app framework is complete
            KeyCode::Char('1') => self.active_tab = MenuTabs::Welcome,
            KeyCode::Char('2') => self.active_tab = MenuTabs::Collection,
            KeyCode::Char('3') => self.active_tab = MenuTabs::Deck,
            KeyCode::Char('4') => self.active_tab = MenuTabs::Help,
            KeyCode::Char('5') => self.active_tab = MenuTabs::Debug,
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    /// exit function
    fn exit(&mut self) {
        self.exit = true
    }
}
