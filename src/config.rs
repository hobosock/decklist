use std::ops::Deref;

use directories_next::ProjectDirs;
use serde::{Deserialize, Serialize};

/// app config settings
#[derive(Debug, Deserialize, Serialize)]
pub struct DecklistConfig {
    pub database_path: String,
    pub database_age_limit: u64,
    pub collection_path: String,
}

impl Default for DecklistConfig {
    fn default() -> Self {
        // TODO: error proof here - I think this just generates a blank
        let project_dir = ProjectDirs::from("", "", "decklist")
            .unwrap()
            .data_local_dir()
            .to_string_lossy()
            .deref()
            .to_string(); // LOL
        Self {
            database_path: project_dir.clone(),
            database_age_limit: 7,
            collection_path: project_dir,
        }
    }
}
