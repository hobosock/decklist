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
use std::{fs, io, thread, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{widgets::ScrollbarState, Frame};
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
    pub filename: Option<String>,
}

impl Default for CollectionMessage {
    fn default() -> Self {
        CollectionMessage {
            debug: String::new(),
            collection: None,
            status: String::new(),
            exist: false,
            filename: None,
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

pub struct App {
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
    pub missing_msg: (
        std::sync::mpsc::Sender<Option<Vec<CollectionCard>>>,
        std::sync::mpsc::Receiver<Option<Vec<CollectionCard>>>,
    ),
    pub missing_check_msg: (
        std::sync::mpsc::Sender<Vec<String>>,
        std::sync::mpsc::Receiver<Vec<String>>,
    ),
    pub waiting_for_missing: bool,
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
    pub missing_lines: Vec<String>,
    pub clipboard: Result<Clipboard, arboard::Error>,
    pub debug_channel: (
        std::sync::mpsc::Sender<String>,
        std::sync::mpsc::Receiver<String>,
    ),
    pub database_ok: bool, // flag to indicate if database_status should be red/green
    pub man_database_file: Option<File>,
    pub man_database_file_name: Option<String>,
    pub prompt_config_update: bool,
}

impl Default for App {
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
            missing_msg: std::sync::mpsc::channel(),
            missing_check_msg: std::sync::mpsc::channel(),
            waiting_for_missing: false,
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
            prompt_config_update: false,
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
                    self.debug_string += "first startup check received\n";
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
                self.debug_string += "starting config check...\n";
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
                    self.debug_string += "config check finished\n";
                    self.config_done = true;
                    self.config_exist = cc.config_exists;
                    self.config_status = cc.config_status;
                    self.config = cc.config.clone();
                    // take care of use_database = false
                    if !cc.config.use_database {
                        self.dc.database_status =
                            "Config set to not use database files.".to_string();
                        self.database_started = true;
                        self.database_done = true;
                        self.dc.need_dl = false;
                        self.dc.ready_load = false;
                    }
                    self.redraw = true;
                }
            }
            // update collection status if no collection file specified in config
            if self.config_done && !self.collection_exist {
                self.collection_status =
                    "No collection file specified in config.  Load manually in [COLLECTION] tab."
                        .to_string();
            }
            if self.config_done
                && self.config.collection_path.is_some()
                && self.collection.is_none()
            {
                self.debug_string += "attempting to auto load collection...\n";
                // if config has a path to a collection and one isn't already loaded, try and load
                // file specified in config
                let collection_channel = self.collection_channel.0.clone();
                let collection_path = self
                    .config
                    .collection_path
                    .clone()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                self.loading_collection = true;
                thread::spawn(move || {
                    let read_result =
                        task::block_on(read_moxfield_collection(collection_path.clone()));
                    let mut message = CollectionMessage::default();
                    match read_result {
                        Ok(collection) => {
                            message.debug += &format!("read {} successfully\n", collection_path);
                            message.collection = Some(collection);
                            message.status =
                                format!("Collection loaded successfully: {}", collection_path);
                            message.exist = true;
                            message.filename = Some(collection_path);
                        }
                        Err(e) => {
                            message.status = e.to_string();
                            message.debug += &format!("Error reading CSV: {}", e);
                        }
                    }
                    if let Ok(()) = collection_channel.send(message) {};
                });
            }
            if self.data_directory_exist && !self.database_started && self.config_done {
                let database_channel = self.database_channel.0.clone();
                let max_age = self.config.database_age_limit;
                let dc_path = self.dc.database_path.clone();
                self.debug_string += &format!(
                    "data directory exists, checking for database on {:?}\n",
                    &dc_path
                );
                thread::spawn(move || {
                    let database_results = task::block_on(database_check(dc_path, max_age));
                    if let Ok(()) = database_channel.send(database_results) {};
                });
                self.database_started = true;
            }
            if !self.database_done {
                if let Ok(dc) = self.database_channel.1.try_recv() {
                    self.debug_string += "received message from database thread\n";
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
                let max_num = self.config.database_num;
                thread::spawn(move || {
                    let database_results = task::block_on(dl_scryfall_latest(dc_clone));
                    let delete_str = match database_management(data_path, max_num) {
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
                self.dc.database_status = format!("Loading {} ...", self.dc.filename);
                let database_channel = self.database_channel.0.clone();
                let dc_clone = self.dc.clone();
                thread::spawn(move || {
                    let database_results = task::block_on(load_database_file(dc_clone));
                    if let Ok(()) = database_channel.send(database_results) {};
                });
                self.dc.ready_load = false;
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
                    self.collection_file_name = msg.filename;
                    self.loading_collection = false;
                    if self.collection.is_some()
                        && self.decklist.is_some()
                        && self.dc.database_cards.is_some()
                        && !self.waiting_for_missing
                    {
                        self.debug_string += "starting missing cards thread...\n";
                        let missing_channel = self.missing_msg.0.clone();
                        let check_channel = self.missing_check_msg.0.clone();
                        let collection = self.collection.clone().unwrap();
                        let decklist = self.decklist.clone().unwrap();
                        let database = self.dc.database_cards.clone().unwrap();
                        thread::spawn(move || {
                            let missing_cards =
                                task::block_on(find_missing_cards(collection, decklist));
                            let mut checks = Vec::new();
                            if missing_cards.is_some() {
                                for card in missing_cards.as_ref().unwrap() {
                                    let missing_str = check_missing(&database, card);
                                    checks.push(format!("{}{}", card, missing_str));
                                }
                            }
                            if let Ok(()) = check_channel.send(checks) {};
                            if let Ok(()) = missing_channel.send(missing_cards) {};
                        });
                        self.waiting_for_missing = true;
                    }
                    self.loading_collection = false;
                    self.redraw = true;
                    // prompt user to save collection path to config
                    self.prompt_config_update = true;
                }
            }
            if self.loading_decklist {
                if let Ok(msg) = self.decklist_channel.1.try_recv() {
                    self.decklist = msg.decklist;
                    self.decklist_status = msg.status;
                    self.loading_decklist = false;
                    if self.collection.is_some()
                        && self.decklist.is_some()
                        && self.dc.database_cards.is_some()
                        && !self.waiting_for_missing
                    {
                        self.debug_string += "starting missing cards thread...\n";
                        let missing_channel = self.missing_msg.0.clone();
                        let check_channel = self.missing_check_msg.0.clone();
                        let collection = self.collection.clone().unwrap();
                        let decklist = self.decklist.clone().unwrap();
                        let database = self.dc.database_cards.clone().unwrap();
                        thread::spawn(move || {
                            let missing_cards =
                                task::block_on(find_missing_cards(collection, decklist));
                            let mut checks = Vec::new();
                            if missing_cards.is_some() {
                                for card in missing_cards.as_ref().unwrap() {
                                    let missing_str = check_missing(&database, card);
                                    checks.push(format!("{}{}", card, missing_str));
                                }
                            }
                            if let Ok(()) = check_channel.send(checks) {};
                            if let Ok(()) = missing_channel.send(missing_cards) {};
                        });
                        self.waiting_for_missing = true;
                    }
                    self.loading_decklist = false;
                    self.redraw = true;
                }
            }
            if self.waiting_for_missing {
                if let Ok(missing_cards) = self.missing_msg.1.try_recv() {
                    self.debug_string += "received missing message\n";
                    self.missing_cards = missing_cards;
                }
                if let Ok(missing_text) = self.missing_check_msg.1.try_recv() {
                    self.debug_string += "received missing text message\n";
                    self.missing_lines = missing_text;
                    self.waiting_for_missing = false;
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
            KeyCode::Char('f') => f_press(self),
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
            // TODO: I think the program can get stuck here if creating one directory fails but the
            // other succeeds - probably need to track each one of these separately
            if !app.directory_exist {
                app.debug_string += "config directory does not exist, attempting to create...\n";
                match create_directory() {
                    Ok(()) => {
                        app.debug_string += "create_directory() succeeded\n";
                        let ok_msg = match app.os {
                            SupportedOS::Linux => "Directory created at ~/.config/decklist",
                            SupportedOS::Windows => "directory created", // TODO: update
                            SupportedOS::Mac => "directory created",
                            SupportedOS::Unsupported => "unknown operating system",
                        };
                        app.directory_status = ok_msg.to_string();
                        app.directory_exist = true;
                    }
                    Err(e) => {
                        app.debug_string += &format!("create_directory() failed: {}", e);
                        app.directory_status = e.to_string();
                    }
                }
            }
            if !app.data_directory_exist {
                app.debug_string += "data directory does not exist, attempting to create...\n";
                match create_data_directory() {
                    Ok(pb) => {
                        app.debug_string += "create_data_directory() succeeded\n";
                        app.data_directory_exist = true;
                        app.dc.database_path = pb;
                    }
                    Err(e) => app.debug_string += &format!("create_data_directory() failed: {}", e),
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
        MenuTabs::Collection => {
            if app.prompt_config_update
                && app.collection_exist
                && app.collection.is_some()
                && app.collection_file.is_some()
            {
                app.config.collection_path = Some(
                    app.collection_file
                        .as_ref()
                        .unwrap()
                        .path()
                        .to_path_buf()
                        .into(),
                );
                let mut config_path = ProjectDirs::from("", "", "decklist")
                    .expect("Should be able to make a config directory in fn c_press().")
                    .config_dir()
                    .to_path_buf();
                config_path.push("config.toml");
                if let Ok(file_text) = toml::to_string(&app.config) {
                    match fs::write(config_path, file_text) {
                        Ok(()) => app.debug_string += "Updated config successfully.\n",
                        Err(e) => app.debug_string += &format!("Config update failed: {}\n", e),
                    }
                } else {
                    app.debug_string += "Failed to convert config struct to TOML text.\n";
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
                    if !app.load_started {
                        app.dc.database_path = path_string.to_path_buf();
                        app.dc.filename = file.name().to_string();
                        // this should trigger the regular database loading process
                        app.dc.ready_load = true;
                        app.load_started = false;
                        app.load_done = false;
                    }
                }
            }
        }
        MenuTabs::Collection => {
            if app.collection_file.is_some() && !app.loading_collection {
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
                                message.filename = Some(path_string);
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
            if app.decklist_file.is_some() && !app.loading_decklist {
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
            // NOTE: prevents autoloading when manually selecting a file
            app.config.collection_path = None;
            app.collection_exist = false;
        }
        MenuTabs::Deck => {
            app.decklist = None;
        }
        _ => {}
    }
}

fn f_press(app: &mut App) {
    match app.active_tab {
        MenuTabs::Missing => {
            if app.missing_cards.is_some()
                && app.decklist_file.is_some()
                && app.decklist_file_name.is_some()
            {
                let mut file_string = String::new();
                for card in app.missing_cards.as_ref().unwrap().iter() {
                    file_string += &format!("{}\n", card);
                }
                if let Some(missing_directory) = app.decklist_file.as_ref().unwrap().path().parent()
                {
                    let missing_filename = missing_directory.to_path_buf().join(&format!(
                        "missing_{}",
                        app.decklist_file_name.as_ref().unwrap()
                    ));
                    app.debug_string += &format!("missing filename: {:?}\n", missing_filename);
                    match fs::write(missing_filename, file_string) {
                        Ok(()) => {
                            app.debug_string +=
                                &format!("Successfully wrote missing cards to file.\n")
                        }
                        Err(e) => {
                            app.debug_string +=
                                &format!("Writing missing cards to file failed: {}\n", e)
                        }
                    }
                }
            }
        }
        _ => {}
    }
}
