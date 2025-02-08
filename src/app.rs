use crate::startup::startup_checks;
use async_std::task;
use std::{io, thread, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{widgets::ScrollbarState, Frame};
use ratatui_explorer::{File, FileExplorer};

use crate::{
    collection::{find_missing_cards, read_decklist, read_moxfield_collection, CollectionCard},
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

pub struct CollectionMessage {
    pub debug: String,
    pub collection: Option<Vec<CollectionCard>>,
    pub status: String,
    pub exist: bool,
}

impl Default for CollectionMessage {
    fn default() -> Self {
        CollectionMessage {
            debug: String::new(),
            collection: None,
            status: String::new(),
            exist: false,
        }
    }
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
    pub missing_cards: Option<Vec<CollectionCard>>,
    pub missing_scroll: usize,
    pub missing_scroll_state: ScrollbarState,
    pub collection_scroll: usize,
    pub collection_scroll_state: ScrollbarState,
    pub decklist_scroll: usize,
    pub decklist_scroll_state: ScrollbarState,
    pub redraw: bool,
    pub collection_channel: (
        std::sync::mpsc::Sender<CollectionMessage>,
        std::sync::mpsc::Receiver<CollectionMessage>,
    ),
    pub decklist_channel: (
        std::sync::mpsc::Sender<Vec<CollectionCard>>,
        std::sync::mpsc::Receiver<Vec<CollectionCard>>,
    ),
    pub loading_collection: bool,
    pub loading_decklist: bool,
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
            missing_cards: None,
            missing_scroll: 0,
            missing_scroll_state: ScrollbarState::default(),
            collection_scroll: 0,
            collection_scroll_state: ScrollbarState::default(),
            decklist_scroll: 0,
            decklist_scroll_state: ScrollbarState::default(),
            redraw: true,
            collection_channel: std::sync::mpsc::channel(),
            decklist_channel: std::sync::mpsc::channel(),
            loading_collection: false,
            loading_decklist: false,
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
        // start new thread to run start up processes
        if !self.startup {
            let startup_channel = self.startup_channel.0.clone();
            thread::spawn(move || {
                let startup_results = task::block_on(startup_checks());
                match startup_channel.send(startup_results) {
                    Ok(()) => {}
                    Err(_) => {}
                }
            });
        }
        // main loop
        while !self.exit {
            // check for startup checks to resolve and update app status
            match self.startup_channel.1.try_recv() {
                Ok(startup_checks) => {
                    // TODO: just add a StartupChecks struct to self struct, pass in one line
                    self.startup = true;
                    self.config_exist = startup_checks.config_exists;
                    self.database_exist = startup_checks.database_exists;
                    self.collection_exist = startup_checks.collection_exists;
                    self.directory_exist = startup_checks.directory_exists;
                    self.data_directory_exist = startup_checks.data_directory_exists;
                    self.database_status = startup_checks.database_status;
                    self.directory_status = startup_checks.directory_status;
                    self.config_status = startup_checks.config_status;
                    self.collection_status = startup_checks.collection_status;
                    self.redraw = true;
                }
                Err(_) => {}
            }
            if self.loading_collection {
                match self.collection_channel.1.try_recv() {
                    Ok(msg) => {
                        self.debug_string += &msg.debug;
                        self.collection = msg.collection;
                        self.collection_status = msg.status;
                        self.collection_exist = msg.exist;
                        self.loading_collection = false;
                        if self.collection.is_some() && self.decklist.is_some() {
                            self.missing_cards = find_missing_cards(
                                self.collection.clone().unwrap(),
                                self.decklist.clone().unwrap(),
                            );
                        }
                        self.redraw = true;
                    }
                    Err(_) => {}
                }
            }
            if self.redraw {
                terminal.draw(|frame| self.render_frame(frame, explorer, explorer2))?;
                self.redraw = false;
            }
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
        if event::poll(Duration::from_millis(100))? {
            let event = event::read()?;
            self.redraw = true;
            match event {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event);
                }
                _ => {}
            };
            if self.active_tab == MenuTabs::Collection && self.collection.is_none() {
                explorer.handle(&event)?;
            }
            if self.active_tab == MenuTabs::Deck && self.decklist.is_none() {
                explorer2.handle(&event)?;
            }
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
            KeyCode::Char('5') => self.active_tab = MenuTabs::Missing,
            KeyCode::Char('6') => self.active_tab = MenuTabs::Help,
            KeyCode::Char('0') => self.active_tab = MenuTabs::Debug,
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('c') => c_press(self),
            KeyCode::Char('s') => s_press(self),
            KeyCode::Enter => enter_press(self),
            KeyCode::Up => up_press(self),
            KeyCode::Down => down_press(self),
            KeyCode::Esc => esc_press(self),
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
        MenuTabs::Missing => {
            if app.missing_cards.is_some() {
                let mut clipboard_string = String::new();
                for card in app.missing_cards.as_ref().unwrap().iter() {
                    clipboard_string += &format!("{}\n", card);
                }
                match cli_clipboard::set_contents(clipboard_string) {
                    Ok(()) => {
                        app.debug_string += "Missing cards copied to clipboard successfully.\n";
                        match cli_clipboard::get_contents() {
                            Ok(contents) => app.debug_string += &(contents + "\n"),
                            Err(e) => app.debug_string += &(e.to_string() + "\n"),
                        }
                    }
                    Err(e) => app.debug_string += &e.to_string(),
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
                let path_str = app.collection_file.as_ref().unwrap().path().to_str();
                if path_str.is_some() {
                    let path_string = path_str.unwrap().to_string();
                    let collection_channel = app.collection_channel.0.clone();
                    app.loading_collection = true;
                    thread::spawn(move || {
                        let read_result =
                            task::block_on(read_moxfield_collection(path_string.clone()));
                        let mut message = CollectionMessage::default();
                        match read_result {
                            Ok(collection) => {
                                message.debug += "read csv successfully\n\n";
                                message.collection = Some(collection);
                                message.status =
                                    format!("Collection loaded successfully: {}", path_string);
                                message.exist = true;
                            }
                            Err(e) => {
                                message.status = e.to_string();
                                message.debug += &format!("Error reading CSV: {}", e);
                            }
                        }
                        match collection_channel.send(message) {
                            Ok(()) => {}
                            Err(_) => {}
                        }
                    });
                } else {
                    app.collection_status = "Encountered error with file path.".to_string();
                }
            } else {
            }
        }
        MenuTabs::Deck => {
            if app.decklist_file.is_some() {
                let path_str = app.decklist_file.as_ref().unwrap().path().to_str();
                if path_str.is_some() {
                    match read_decklist(path_str.unwrap()) {
                        Ok(decklist) => {
                            app.decklist = Some(decklist);
                            app.decklist_status =
                                format!("Decklist loaded successfully: {}", path_str.unwrap());
                            if app.collection.is_some() && app.decklist.is_some() {
                                app.missing_cards = find_missing_cards(
                                    app.collection.clone().unwrap(),
                                    app.decklist.clone().unwrap(),
                                );
                            }
                        }
                        Err(e) => {
                            app.decklist_status = e.to_string();
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn up_press(app: &mut App) {
    match app.active_tab {
        MenuTabs::Collection => {
            if app.collection_scroll > 0 {
                app.collection_scroll -= 1;
                app.collection_scroll_state =
                    app.collection_scroll_state.position(app.collection_scroll);
            }
        }
        MenuTabs::Deck => {
            if app.decklist_scroll > 0 {
                app.decklist_scroll -= 1;
                app.decklist_scroll_state = app.decklist_scroll_state.position(app.decklist_scroll);
            }
        }
        MenuTabs::Missing => {
            if app.missing_scroll > 0 {
                app.missing_scroll -= 1;
                app.missing_scroll_state = app.missing_scroll_state.position(app.missing_scroll);
            }
        }
        _ => {}
    }
}

fn down_press(app: &mut App) {
    match app.active_tab {
        MenuTabs::Collection => {
            if app.collection.is_some()
                && app.collection_scroll < app.collection.as_ref().unwrap().len()
            {
                app.collection_scroll += 1;
                app.collection_scroll_state =
                    app.collection_scroll_state.position(app.collection_scroll);
            }
        }
        MenuTabs::Deck => {
            if app.decklist.is_some() && app.decklist_scroll < app.decklist.as_ref().unwrap().len()
            {
                app.decklist_scroll += 1;
                app.decklist_scroll_state = app.decklist_scroll_state.position(app.decklist_scroll);
            }
        }
        MenuTabs::Missing => {
            if app.missing_cards.is_some()
                && app.missing_scroll < app.missing_cards.as_ref().unwrap().len()
            {
                app.missing_scroll += 1;
                app.missing_scroll_state = app.missing_scroll_state.position(app.missing_scroll);
            }
        }
        _ => {}
    }
}

fn esc_press(app: &mut App) {
    match app.active_tab {
        MenuTabs::Collection => {
            app.collection = None;
            app.collection_exist = false;
        }
        MenuTabs::Deck => {
            app.decklist = None;
        }
        _ => {}
    }
}
