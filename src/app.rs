use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use futures::join;
use ratatui::Frame;
use ratatui_explorer::{File, FileExplorer};

use crate::{
    collection::{read_moxfield_collection, CollectionCard},
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
    pub collection: Option<Vec<CollectionCard>>,
    pub collection_file_name: Option<String>,
    pub collection_file: Option<File>,
    pub decklist: Option<Vec<CollectionCard>>,
    pub decklist_file_name: Option<String>,
    pub decklist_file: Option<File>,
    pub decklist_status: String,
    pub debug_string: String,
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
            collection: None,
            collection_file_name: None,
            collection_file: None,
            decklist: None,
            decklist_file_name: None,
            decklist_file: None,
            decklist_status: String::new(),
            debug_string: String::new(),
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
        if self.active_tab == MenuTabs::Collection {
            explorer.handle(&event)?;
        }
        if self.active_tab == MenuTabs::Deck {
            explorer2.handle(&event)?;
        }
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
            KeyCode::Char('s') => s_press(self),
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

fn s_press(app: &mut App) {
    match app.active_tab {
        MenuTabs::Collection => {
            if app.collection_file.is_some() {
                app.debug_string += "Collection file is some.\n";
                let path_str = app.collection_file.as_ref().unwrap().path().to_str();
                if path_str.is_some() {
                    app.debug_string += "path_str is some\n";
                    match read_moxfield_collection(path_str.unwrap()) {
                        Ok(collection) => {
                            app.debug_string += "read csv successfully\n\n";
                            app.debug_string += &format!("{:?}", collection);
                            app.collection = Some(collection);
                            app.collection_status =
                                format!("Collection loaded successfully: {}", path_str.unwrap());
                        }
                        Err(e) => {
                            app.collection_status = e.to_string();
                            app.debug_string += &format!("Error reading CSV: {}", e);
                        }
                    }
                } else {
                    app.collection_status = "Encountered error with file path.".to_string();
                    app.debug_string += "path_str is none\n"
                }
            } else {
                app.debug_string += "Collection file is none.\n";
            }
        }
        MenuTabs::Deck => {}
        _ => {}
    }
}
