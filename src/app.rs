use crate::{
    collection::check_missing,
    startup::{
        config_check, database_check, database_management, directory_check, dl_scryfall_latest,
        load_database_file, ConfigCheck, DatabaseCheck, DirectoryCheck,
    },
};
use arboard::Clipboard;
use async_std::task;
use directories_next::ProjectDirs;
use std::{io, thread, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{text::Line, widgets::ScrollbarState, Frame};
use ratatui_explorer::{File, FileExplorer};

use crate::{
    collection::{find_missing_cards, read_decklist, read_moxfield_collection, CollectionCard},
    config::DecklistConfig,
    startup::{create_config, create_data_directory, create_directory},
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

pub struct DecklistMessage {
    pub decklist: Option<Vec<CollectionCard>>,
    pub status: String,
}

impl Default for DecklistMessage {
    fn default() -> Self {
        DecklistMessage {
            decklist: None,
            status: String::new(),
        }
    }
}

pub struct App<'a> {
    exit: bool,
    pub startup: bool,
    pub config_started: bool,
    pub config_done: bool,
    pub database_started: bool,
    pub database_done: bool,
    pub dc_started: bool,
    pub dl_started: bool,
    pub dl_done: bool,
    pub load_started: bool,
    pub load_done: bool,
    pub os: SupportedOS,
    pub directory_exist: bool,
    pub data_directory_exist: bool,
    pub config_exist: bool,
    pub collection_exist: bool,
    pub directory_status: String,
    pub config_status: String,
    pub collection_status: String,
    pub config: DecklistConfig,
    pub active_tab: MenuTabs,
    pub directory_channel: (
        std::sync::mpsc::Sender<DirectoryCheck>,
        std::sync::mpsc::Receiver<DirectoryCheck>,
    ),
    pub config_channel: (
        std::sync::mpsc::Sender<ConfigCheck>,
        std::sync::mpsc::Receiver<ConfigCheck>,
    ),
    pub database_channel: (
        std::sync::mpsc::Sender<DatabaseCheck>,
        std::sync::mpsc::Receiver<DatabaseCheck>,
    ),
    pub dc: DatabaseCheck,
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
        std::sync::mpsc::Sender<DecklistMessage>,
        std::sync::mpsc::Receiver<DecklistMessage>,
    ),
    pub loading_collection: bool,
    pub loading_decklist: bool,
    pub missing_lines: Vec<Line<'a>>,
    pub clipboard: Result<Clipboard, arboard::Error>,
    pub debug_channel: (
        std::sync::mpsc::Sender<String>,
        std::sync::mpsc::Receiver<String>,
    ),
    pub database_ok: bool, // flag to indicate if database_status should be red/green
    pub man_database_file: Option<File>,
    pub man_database_file_name: Option<String>,
}

impl Default for App<'_> {
    fn default() -> Self {
        Self {
            exit: false,
            startup: false,
            config_started: false,
            config_done: false,
            database_started: false,
            database_done: false,
            dc_started: false,
            dl_started: false,
            dl_done: false,
            load_started: false,
            load_done: false,
            os: SupportedOS::default(),
            directory_exist: false,
            data_directory_exist: false,
            config_exist: false,
            collection_exist: false,
            directory_status: "Waiting on startup checks...".to_string(),
            config_status: "Waiting on startup checks...".to_string(),
            collection_status: "Waiting on startup checks...".to_string(),
            config: DecklistConfig::default(),
            active_tab: MenuTabs::default(),
            directory_channel: std::sync::mpsc::channel(),
            config_channel: std::sync::mpsc::channel(),
            database_channel: std::sync::mpsc::channel(),
            dc: DatabaseCheck::default(),
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
            missing_lines: Vec::new(),
            clipboard: Clipboard::new(),
            debug_channel: std::sync::mpsc::channel(),
            database_ok: false,
            man_database_file: None,
            man_database_file_name: None,
        }
    }
}

impl App<'_> {
    /// runs the application's main loop until the user quits
    pub fn run(
        &mut self,
        terminal: &mut Tui,
        explorer: &mut FileExplorer,
        explorer2: &mut FileExplorer,
        database_explorer: &mut FileExplorer,
    ) -> io::Result<()> {
        // start new threads to run start up processes
        if !self.startup && !self.dc_started {
            let directory_channel = self.directory_channel.0.clone();
            thread::spawn(move || {
                let directory_results = task::block_on(directory_check());
                if let Ok(()) = directory_channel.send(directory_results) {};
            });
            self.dc_started = true;
        }
        // main loop
        while !self.exit {
            if !self.startup {
                // check for startup checks to resolve and update app status
                if let Ok(directory_check) = self.directory_channel.1.try_recv() {
                    self.startup = true;
                    self.directory_exist = directory_check.directory_exists;
                    self.data_directory_exist = directory_check.data_directory_exists;
                    self.directory_status = directory_check.directory_status;
                    self.redraw = true;
                    self.dc.database_path = directory_check.data_path; // TODO: do config also
                }
            }
            // spin up other startup processes once directories have been confirmed
            if self.directory_exist && !self.config_started {
                let project_dir = ProjectDirs::from("", "", "decklist").unwrap();
                let config_channel = self.config_channel.0.clone();
                thread::spawn(move || {
                    let config_results = task::block_on(config_check(project_dir));
                    if let Ok(()) = config_channel.send(config_results) {};
                });
                self.config_started = true;
            }
            if !self.config_done {
                if let Ok(cc) = self.config_channel.1.try_recv() {
                    self.config_done = true;
                    self.config_exist = cc.config_exists;
                    self.config_status = cc.config_status;
                    self.redraw = true;
                }
            }
            if self.data_directory_exist && !self.database_started && self.config_done {
                let database_channel = self.database_channel.0.clone();
                let max_age = self.config.database_age_limit;
                let dc_path = self.dc.database_path.clone();
                thread::spawn(move || {
                    let database_results = task::block_on(database_check(dc_path, max_age));
                    if let Ok(()) = database_channel.send(database_results) {};
                });
                self.database_started = true;
            }
            if !self.database_done {
                if let Ok(dc) = self.database_channel.1.try_recv() {
                    self.database_done = true;
                    self.dc.database_exists = dc.database_exists;
                    self.dc.database_status = dc.database_status;
                    self.dc.database_cards = dc.database_cards;
                    self.dc.need_dl = dc.need_dl;
                    self.dc.ready_load = dc.ready_load;
                    self.dc.filename = dc.filename;
                    if dc.database_exists {
                        self.database_ok = true;
                    }
                    self.collection_exist = false;
                    self.collection_status =
                        "Manually load in [COLLECTION] tab until feature is added.".to_string();
                    self.redraw = true;
                }
            }
            if self.dc.need_dl && !self.dl_started {
                let database_channel = self.database_channel.0.clone();
                // NOTE: it might seem like a waste to copy the whole database vector, but it
                // should be None still - nothing has been loaded yet
                let dc_clone = self.dc.clone();
                let debug_channel = self.debug_channel.0.clone();
                let data_path = self.dc.database_path.clone();
                thread::spawn(move || {
                    let database_results = task::block_on(dl_scryfall_latest(dc_clone));
                    let delete_str = match database_management(data_path) {
                        Ok(()) => "File deleted successfully.\n".to_string(),
                        Err(e) => e.to_string(),
                    };
                    if let Ok(()) = database_channel.send(database_results) {};
                    if let Ok(()) = debug_channel.send(delete_str) {};
                });
                self.dl_started = true;
            }
            if self.dl_started && !self.dl_done {
                if let Ok(dc) = self.database_channel.1.try_recv() {
                    self.debug_string += "received response from download thread!\n";
                    self.dc = dc;
                    self.database_ok = self.dc.ready_load; // status goes red if DL failed or
                    self.dl_done = true;
                    self.dl_started = false;
                    self.redraw = true
                }
                if let Ok(s) = self.debug_channel.1.try_recv() {
                    self.debug_string += &s;
                }
            }
            if self.dc.ready_load && !self.load_started {
                self.debug_string += &format!(
                    "Loading a database file : {:?}{}\n",
                    self.dc.database_path.display(),
                    self.dc.filename
                );
                let database_channel = self.database_channel.0.clone();
                let dc_clone = self.dc.clone();
                thread::spawn(move || {
                    let database_results = task::block_on(load_database_file(dc_clone));
                    if let Ok(()) = database_channel.send(database_results) {};
                });
                self.load_started = true;
            }
            if self.load_started && !self.load_done {
                if let Ok(dc) = self.database_channel.1.try_recv() {
                    self.dc = dc;
                    self.load_done = true;
                    self.load_started = false;
                    if self.dc.database_cards.is_some() {
                        self.database_ok = true;
                    } else {
                        self.database_ok = false;
                    }
                    self.redraw = true;
                }
            }
            if self.loading_collection {
                if let Ok(msg) = self.collection_channel.1.try_recv() {
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
                        self.missing_lines = Vec::new();
                        for card in self.missing_cards.clone().unwrap() {
                            let missing_str = if self.dc.database_cards.is_some() {
                                check_missing(self.dc.database_cards.as_ref().unwrap(), &card)
                            } else {
                                "".to_string()
                            };
                            self.missing_lines
                                .push(Line::from(format!("{}{}", card, missing_str)));
                        }
                    }
                    self.redraw = true;
                }
            }
            if self.loading_decklist {
                if let Ok(msg) = self.decklist_channel.1.try_recv() {
                    self.decklist = msg.decklist;
                    self.decklist_status = msg.status;
                    self.loading_decklist = false;
                    if self.collection.is_some() && self.decklist.is_some() {
                        self.missing_cards = find_missing_cards(
                            self.collection.clone().unwrap(),
                            self.decklist.clone().unwrap(),
                        );
                        self.missing_lines = Vec::new();
                        for card in self.missing_cards.clone().unwrap() {
                            let missing_str = if self.dc.database_cards.is_some() {
                                check_missing(self.dc.database_cards.as_ref().unwrap(), &card)
                            } else {
                                "".to_string()
                            };
                            self.missing_lines
                                .push(Line::from(format!("{}{}", card, missing_str)));
                        }
                    }
                    self.redraw = true;
                }
            }
            if self.redraw {
                terminal.draw(|frame| {
                    self.render_frame(frame, explorer, explorer2, database_explorer)
                })?;
                self.redraw = false;
            }
            self.handle_events(explorer, explorer2, database_explorer)?;
        }
        Ok(())
    }

    /// render the frame
    fn render_frame(
        &mut self,
        frame: &mut Frame,
        explorer: &mut FileExplorer,
        explorer2: &mut FileExplorer,
        database_explorer: &mut FileExplorer,
    ) {
        ui(frame, self, explorer, explorer2, database_explorer);
    }

    /// updates application state based on user input
    fn handle_events(
        &mut self,
        explorer: &mut FileExplorer,
        explorer2: &mut FileExplorer,
        database_explorer: &mut FileExplorer,
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
            if self.active_tab == MenuTabs::Database {
                database_explorer.handle(&event)?;
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
                app.debug_string += &clipboard_string;
                if app.clipboard.is_ok() {
                    let clipboard = app.clipboard.as_mut().unwrap();
                    match clipboard.set_text(clipboard_string) {
                        Ok(()) => {
                            app.debug_string += "Missing cards copied to clipboard successfully.\n";
                            match clipboard.get_text() {
                                Ok(contents) => app.debug_string += &(contents + "\n"),
                                Err(e) => app.debug_string += &(e.to_string() + "\n"),
                            }
                        }
                        Err(e) => app.debug_string += &e.to_string(),
                    }
                } else {
                    app.debug_string += "Clipboard was not successfully created.\n";
                }
            }
        }
        _ => {}
    }
}

fn s_press(app: &mut App) {
    match app.active_tab {
        MenuTabs::Database => {
            if let Some(file) = &app.man_database_file {
                if let Some(path_string) = file.path().parent() {
                    app.dc.database_path = path_string.to_path_buf();
                    app.dc.filename = file.name().to_string();
                    // this should trigger the regular database loading process
                    app.dc.ready_load = true;
                    app.load_started = false;
                    app.load_done = false;
                }
            }
        }
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
                        if let Ok(()) = collection_channel.send(message) {};
                    });
                } else {
                    app.collection_status = "Encountered error with file path.".to_string();
                }
            }
        }
        MenuTabs::Deck => {
            if app.decklist_file.is_some() {
                let path_str = app.decklist_file.as_ref().unwrap().path().to_str();
                if path_str.is_some() {
                    let path_string = path_str.unwrap().to_string();
                    let decklist_channel = app.decklist_channel.0.clone();
                    app.loading_decklist = true;
                    thread::spawn(move || {
                        let read_result = read_decklist(path_string.clone());
                        let mut message = DecklistMessage::default();
                        match read_result {
                            Ok(decklist) => {
                                message.decklist = Some(decklist);
                                message.status =
                                    format!("Decklist loaded successfully: {}", path_string);
                            }
                            Err(e) => {
                                message.status = e.to_string();
                            }
                        }
                        if let Ok(()) = decklist_channel.send(message) {};
                    });
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
