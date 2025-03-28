use std::path::Path;

use directories_next::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::database::scryfall::PriceType;

/// app config settings
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DecklistConfig {
    pub use_database: bool, // set to false to never download or auto load a database file
    pub database_path: Box<Path>, // folder containing downloaded database files
    pub database_age_limit: u64, // age in days before a new database file is downloaded
    pub database_num: u64,  // number of database files to keep around
    pub collection_path: Option<Box<Path>>, // path to latest collection file
    pub currency: PriceType,
}

impl Default for DecklistConfig {
    fn default() -> Self {
        let data_dir = ProjectDirs::from("", "", "decklist")
            .expect("Failed to create a project directory name.  IDK what to do here.")
            .data_local_dir()
            .to_path_buf();
        Self {
            use_database: true,
            database_path: data_dir.clone().into(),
            database_age_limit: 7,
            database_num: 3,
            collection_path: None,
            currency: PriceType::USD,
        }
    }
}
