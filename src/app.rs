use std::io;

use async_std::fs::create_dir;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::Frame;
use ratatui_explorer::FileExplorer;

use crate::{
    config::DecklistConfig,
    database::scryfall::ScryfallCard,
    startup::{create_config, create_data_directory, create_directory, StartupChecks},
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
    pub data_directory_exist: bool,
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
    pub card_database: Vec<ScryfallCard>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            exit: false,
            startup: false,
            os: SupportedOS::default(),
            directory_exist: false,
            data_directory_exist: false,
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
            card_database: Vec::new(),
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
            KeyCode::Char('2') => self.active_tab = MenuTabs::Database,
            KeyCode::Char('3') => self.active_tab = MenuTabs::Collection,
            KeyCode::Char('4') => self.active_tab = MenuTabs::Deck,
            KeyCode::Char('5') => self.active_tab = MenuTabs::Help,
            KeyCode::Char('0') => self.active_tab = MenuTabs::Debug,
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('c') => c_press(self),
            KeyCode::Enter => enter_press(self),
            _ => {}
        }
    }

    /// exit function
    fn exit(&mut self) {
        self.exit = true
    }
}

fn enter_press(app: &mut App) {
    match app.active_tab {
        MenuTabs::Welcome => {
            if !app.directory_exist {
                match create_directory() {
                    Ok(()) => {
                        let ok_msg = match app.os {
                            SupportedOS::Linux => "Directory created at ~/.config/decklist",
                            SupportedOS::Windows => "directory created", // TODO: update
                            SupportedOS::Mac => "directory created",
                            SupportedOS::Unsupported => "unknown operating system",
                        };
                        app.directory_status = ok_msg.to_string();
                        app.directory_exist = true;
                    }
                    Err(e) => app.directory_status = e.to_string(),
                }
            }
            if !app.data_directory_exist {
                match create_data_directory() {
                    Ok(()) => {
                        app.data_directory_exist = true;
                    } // TODO: message?
                    Err(_e) => {}
                }
            }
        }
        _ => {}
    }
}

fn c_press(app: &mut App) {
    match app.active_tab {
        MenuTabs::Welcome => {
            if !app.config_exist {
                match create_config() {
                    Ok(()) => {
                        let ok_msg = match app.os {
                            SupportedOS::Linux => {
                                "Config created at ~/.config/decklist/config.toml"
                            }
                            SupportedOS::Windows => "directory created", // TODO: update
                            SupportedOS::Mac => "directory created",
                            SupportedOS::Unsupported => "unknown operating system",
                        };
                        app.config_status = ok_msg.to_string();
                        app.config_exist = true;
                    }
                    Err(e) => app.config_status = e.to_string(),
                }
            }
        }
        _ => {}
    }
}
