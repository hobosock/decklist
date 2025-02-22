use std::{
    error::Error,
    ffi::OsString,
    fs::{self, create_dir},
    io::Write,
    path::PathBuf,
    time::Duration,
};

use chrono::{DateTime, Local};
use directories_next::ProjectDirs;
use serde::Deserialize;
use ureq::Agent;

use crate::{
    config::DecklistConfig,
    database::scryfall::{read_scryfall_database, ScryfallCard},
};

/*
/// returns enum of detected OS
/// useful for matching system specific file paths, config files, etc.
pub fn get_os() -> SupportedOS {
    let os_string = std::env::consts::OS;
    if os_string == "linux" {
        SupportedOS::Linux
    } else if os_string == "windows" {
        SupportedOS::Windows
    } else if os_string == "macos" {
        SupportedOS::Mac
    } else {
        SupportedOS::Unsupported
    }
}
*/

/// struct with information resulting from directory startup check
pub struct DirectoryCheck {
    pub directory_exists: bool,
    pub data_directory_exists: bool,
    pub directory_status: String,
    pub config_path: PathBuf,
    pub data_path: PathBuf,
}

/// checks for existence of program directories that store config file/database/etc.
/// first step in startup checks
pub async fn directory_check() -> DirectoryCheck {
    let directory_exists = directory_exist();
    let data_directory_exists = data_directory_exist();
    let mut config_path = PathBuf::new();
    let mut data_path = PathBuf::new();
    let mut directory_status =
        "Program directories does not exist.  Hit enter to create them now.".to_string();
    if directory_exists {
        let project_dir = ProjectDirs::from("", "", "decklist").unwrap();
        directory_status = format!(
            "Directory found at {:?}",
            project_dir.config_dir().to_path_buf()
        );
        config_path = project_dir.config_dir().to_owned();
        data_path = project_dir.data_local_dir().to_owned();
    }
    DirectoryCheck {
        directory_exists,
        data_directory_exists,
        directory_status,
        config_path,
        data_path,
    }
}

/// struct with config check information
pub struct ConfigCheck {
    pub config_exists: bool,
    pub config_status: String,
    pub config: DecklistConfig,
}

/// checks for existence of config file, loads if one is found
/// returns default options if no file is found or if an error occurs
/// NOTE: check that directory exists first before calling this one
pub async fn config_check(project_dir: ProjectDirs) -> ConfigCheck {
    let mut config_exists = false;
    let mut config = DecklistConfig::default();
    let mut config_status = "No directory for config file.".to_string();
    match config_exist(project_dir) {
        Ok(c) => {
            config = c;
            config_status = "Config successfully loaded.".to_string();
            config_exists = true;
        }
        Err(e) => {
            config_status = format!(
                "Failed to load config file: {}.  Press C to create.  Using default settings...",
                e
            );
        }
    }
    ConfigCheck {
        config_exists,
        config_status,
        config,
    }
}

/// struct with database check information
#[derive(Clone)]
pub struct DatabaseCheck {
    pub database_exists: bool,
    pub database_status: String,
    pub database_cards: Option<Vec<ScryfallCard>>,
    pub database_path: PathBuf,
    pub filename: String,
    pub need_dl: bool,
    pub ready_load: bool,
}

impl Default for DatabaseCheck {
    fn default() -> Self {
        DatabaseCheck {
            database_exists: false,
            database_status: "Waiting on startup checks...".to_string(),
            database_cards: None,
            database_path: PathBuf::new(),
            filename: String::new(),
            need_dl: false,
            ready_load: false,
        }
    }
}

/// checks for existence of database file
pub async fn database_check(data_path: PathBuf, max_age: u64) -> DatabaseCheck {
    let mut need_download = false;
    let mut ready_load = false;
    let mut database_exists = false;
    let mut database_status =
        "Failed to find a valid database file.  Load file manually in [DATABASE] tab.".to_string();
    let database_cards = None;
    let mut filename = String::new();
    if let Some((fname, date)) = find_scryfall_database(data_path.clone()) {
        let current_time: DateTime<Local> = Local::now();
        let formatted_time = current_time.format("%Y%m%d%H%M%S").to_string();
        let time_num = match formatted_time.parse::<u64>() {
            Ok(n) => n,
            Err(_) => 0,
        };
        if (time_num - date) > (max_age * 1000000) {
            // NOTE: HHMMSS place
            need_download = true;
            database_status = format!(
                "Database file found, but it is older than {} days.  Downloading new file...",
                max_age
            );
            filename = fname; // NOTE: go ahead and load latest filename in case DL fails
        } else {
            database_status = format!("Recent database file found: {}", fname.clone());
            filename = fname;
            ready_load = true;
        }
        database_exists = true;
    }
    DatabaseCheck {
        database_exists,
        database_status,
        database_cards,
        database_path: data_path,
        need_dl: need_download,
        ready_load,
        filename,
    }
}

/// attempts to load given database file
/// updates status accordingly
pub async fn load_database_file(mut dc: DatabaseCheck) -> DatabaseCheck {
    let mut data_path = dc.database_path.clone();
    data_path.push(dc.filename.clone());
    match read_scryfall_database(&data_path) {
        Ok(cards) => {
            dc.database_exists = true;
            dc.database_status = "Loaded cards".to_string();
            dc.database_cards = Some(cards);
        }
        Err(e) => dc.database_status = e.to_string(),
    }
    dc
}

/// finds the latest Scryfall OracleCards database file in the program data directory
/// returns the full file path as Some(String) if found
/// returns None if no file exists
fn find_scryfall_database(data_path: PathBuf) -> Option<(String, u64)> {
    let items = fs::read_dir(data_path)
        .expect("Scryfall database directory should exist if calling find_scryfall_database().");
    let mut options = Vec::new();
    let mut dates = Vec::new();
    for item in items {
        match item {
            Ok(f) => {
                let f_str = f.file_name().into_string();
                if f_str.is_ok() && f_str.as_ref().unwrap().contains("oracle-cards") {
                    let f_string = f_str.unwrap();
                    options.push(f_string.clone());
                    let sections: Vec<&str> = f_string.split("-").collect();
                    let subsections: Vec<&str> = sections[2].split('.').collect(); // ######.json
                    match subsections[0].trim().parse::<u64>() {
                        Ok(num) => dates.push(num),
                        Err(_) => dates.push(0),
                    }
                }
            }
            Err(_) => {}
        }
    }
    let result = match dates.iter().enumerate().max() {
        Some((position, date)) => Some((options[position].clone(), *date)),
        None => None,
    };
    result
}

/// downloads latest OracleCards bulk data from Scryfall
pub async fn dl_scryfall_latest(mut dc: DatabaseCheck) -> DatabaseCheck {
    match scryfall_bulk_request(dc.database_path.clone()).await {
        Ok(filename) => {
            dc.filename = filename.clone();
            dc.database_status = format!("JSON successfully downloaded: {}", filename);
            dc.ready_load = true;
            dc.need_dl = false;
        }
        Err(e) => {
            if dc.filename.is_empty() {
                // no previous file available
                dc.database_status = format!("Failed to download file from Scryfall: {}", e);
                dc.need_dl = false; // don't try to download again
                dc.ready_load = false;
            } else {
                // try and load existing, out of date file
                dc.database_status = format!(
                    "Failed to download a new file from Scryfall: {}.  Using exisint file: {}",
                    e,
                    dc.filename.clone()
                );
                dc.ready_load = true;
                dc.need_dl = false;
            }
        }
    }
    dc
}

/// makes http requests to get latest bulk data from Scryfall
async fn scryfall_bulk_request(
    mut data_path: PathBuf,
) -> Result<String, Box<dyn std::error::Error>> {
    let config = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .build();
    let scryfall_agent: Agent = config.into();

    // first request gets URI for latest data
    let resp: ScryfallResponse = scryfall_agent
        .get("https://api.scryfall.com/bulk-data/oracle-cards")
        .header("User-Agent", "decklistv0.1")
        .header("Accept", "*/*")
        .call()?
        .body_mut()
        .read_json::<ScryfallResponse>()?;

    // make filename from latest URI
    let uri = resp.download_uri.clone();
    let uri_pieces: Vec<&str> = resp.download_uri.split("/").collect();
    let name = uri_pieces[uri_pieces.len() - 1];
    data_path.push(name);

    // second request downloads JSON file to user data directory
    let download_request = scryfall_agent
        .get(uri)
        .header("User-Agent", "decklistv0.1")
        .header("Accept", "application/file")
        .call()?
        .body_mut()
        .with_config()
        .limit(resp.size)
        .read_to_string()?;
    fs::write(data_path, &download_request)?;

    Ok(name.to_string())
}

#[derive(Deserialize, Debug)]
struct ScryfallResponse {
    object: String,
    id: String,
    r#type: String,
    updated_at: String,
    uri: String,
    name: String,
    description: String,
    size: u64,
    download_uri: String,
    content_type: String,
    content_encoding: String,
}

/// checks for existence of decklist user directory
fn directory_exist() -> bool {
    let mut result = false;
    if let Some(project_dir) = ProjectDirs::from("", "", "decklist") {
        let path = project_dir.config_dir();
        result = path.exists();
    }
    result
}

/// checks for existence of decklist data directory
fn data_directory_exist() -> bool {
    let mut result = false;
    if let Some(project_dir) = ProjectDirs::from("", "", "decklist") {
        let path = project_dir.data_local_dir();
        result = path.exists();
    }
    result
}

/// creates config directory if it doesn't exist
pub fn create_directory() -> Result<(), std::io::Error> {
    let mut result = Err(std::io::Error::last_os_error()); // TODO: sucks
    if let Some(project_dir) = ProjectDirs::from("", "", "decklist") {
        let path = project_dir.config_dir();
        result = create_dir(path);
    }
    result
}

/// creates data directory if it doesn't exist
pub fn create_data_directory() -> Result<(), std::io::Error> {
    let mut result = Err(std::io::Error::last_os_error());
    if let Some(data_dir) = ProjectDirs::from("", "", "decklist") {
        let path = data_dir.data_local_dir();
        result = create_dir(path);
    }
    result
}

/// checks for existance of a config.toml
fn config_exist(dir: ProjectDirs) -> Result<DecklistConfig, Box<dyn Error>> {
    let mut config_path = dir.config_dir().as_os_str().to_os_string();
    config_path.push("/config.toml");
    let read_result = fs::read_to_string(config_path)?;
    let config: DecklistConfig = toml::from_str(&read_result)?;
    Ok(config)
}

/// creates a default config.toml file
pub fn create_config() -> Result<(), Box<dyn Error>> {
    // NOTE: using unwrap() because directory should exist before this function can be called
    let mut config_path = ProjectDirs::from("", "", "decklist")
        .unwrap()
        .config_dir()
        .as_os_str()
        .to_os_string();
    config_path.push("/config.toml");
    let default_text = toml::to_string(&DecklistConfig::default())?;
    Ok(fs::write(config_path, default_text)?)
}
