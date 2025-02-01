use std::{error::Error, fs::File};

use csv::ReaderBuilder;
use serde::Deserialize;

/// simple card format for collections and decklists
/// just the card name and the quantity
#[derive(Deserialize)]
pub struct CollectionCard {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Count")]
    pub quantity: u64,
}

/// reads in Moxfield collection CSV and turns it into a Vec<CollectionCard>
pub fn read_moxfield_collection(file_name: &str) -> Result<Vec<CollectionCard>, Box<dyn Error>> {
    let file = File::open(file_name)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_reader(file);
    let mut iter = reader.deserialize();
    let mut collection = Vec::new();
    if let Some(result) = iter.next() {
        let record: CollectionCard = result?;
        collection.push(record);
    }
    Ok(collection)
}
