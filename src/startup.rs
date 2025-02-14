use std::{
    error::Error,
    fs::{self, create_dir},
};

use directories_next::ProjectDirs;

use crate::{
    app::SupportedOS,
    config::DecklistConfig,
    database::scryfall::{read_scryfall_database, ScryfallCard},
};

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

/// structure containing the results of all the different startup checks
/// to be passed back to main application thread and used to update main app struct
pub struct StartupChecks {
    pub directory_exists: bool,
    pub data_directory_exists: bool,
    pub config_exists: bool,
    pub database_exists: bool,
    pub collection_exists: bool,
    pub config: DecklistConfig,
    pub directory_status: String,
    pub config_status: String,
    pub database_status: String,
    pub collection_status: String,
    pub database_cards: Option<Vec<ScryfallCard>>,
}

/// checks for supported OS, then looks at OS appropriate file locations
/// for things like config file/database/collection locations
pub async fn startup_checks() -> StartupChecks {
    let os = get_os();
    let directory_exists = directory_exist();
    let data_directory_exists = data_directory_exist();
    let mut config_exists = false;
    let mut config = DecklistConfig::default();
    let mut directory_status = "directory does not exist.  Hit enter to create it now.".to_string();
    let mut config_status = "No directory for config file.".to_string();
    let mut database_exists = false;
    let mut database_status =
        "No config file to indicate database location.  Load file manually in [DATABASE] tab."
            .to_string();
    let mut database_cards = None;
    let mut collection_status =
        "No config file to indicate collection location.  Load file manually in [COLLECTION] tab."
            .to_string();
    if directory_exists {
        let project_dir = ProjectDirs::from("", "", "decklist").unwrap();
        directory_status = format!(
            "Directory found at {:?}",
            project_dir.config_dir().to_path_buf()
        );
        match config_exist(project_dir.clone()) {
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
        // TODO: check database exist/age/etc.
        let mut data_path = project_dir.data_local_dir().as_os_str().to_os_string();
        data_path.push("/short.json");
        match read_scryfall_database(&data_path) {
            Ok(cards) => {
                database_exists = true;
                database_status = "Loaded cards".to_string();
                database_cards = Some(cards);
            }
            Err(e) => database_status = e.to_string(),
        }
    }

    StartupChecks {
        directory_exists,
        data_directory_exists,
        config_exists,
        database_exists,
        collection_exists: false,
        config,
        directory_status,
        config_status,
        database_status,
        collection_status,
        database_cards,
    }
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
