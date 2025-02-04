use std::{
    error::Error,
    fs::{self, File},
};

use csv::ReaderBuilder;
use serde::Deserialize;

/// simple card format for collections and decklists
/// just the card name and the quantity
#[derive(Deserialize, Clone, Debug)]
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
    let iter = reader.deserialize();
    let mut collection = Vec::new();
    for result in iter {
        let record: CollectionCard = result?;
        collection.push(record);
    }
    let squashed = squash_collection(collection);
    Ok(squashed)
}

/// Moxfield collection treats different printings of the same card as individual line items, but
/// that's not a distinction we need for this program.  This function "squashes" the quantities of
/// all printings of the same card into a single line item.
fn squash_collection(collection_in: Vec<CollectionCard>) -> Vec<CollectionCard> {
    let mut collection_out: Vec<CollectionCard> = Vec::new();
    for card in collection_in.iter() {
        if collection_out.len() > 0 {
            // check if name already exists
            let mut matched = false;
            let mut bonus = 0;
            let mut index = 0;
            for (i, squashed) in collection_out.iter().enumerate() {
                if card.name == squashed.name {
                    matched = true;
                    bonus = card.quantity;
                    index = i;
                    break;
                }
            }
            if !matched {
                collection_out.push(card.clone());
            } else {
                collection_out[index].quantity += bonus;
            }
        } else {
            // go ahead and copy it over
            collection_out.push(card.clone());
        }
    }
    collection_out
}

pub fn read_decklist(file_name: &str) -> Result<Vec<CollectionCard>, Box<dyn Error>> {
    let mut decklist: Vec<CollectionCard> = Vec::new();
    let file_str = fs::read_to_string(file_name)?;
    let rows: Vec<&str> = file_str.split('\n').collect();
    for line in rows.iter() {
        // check for "Sideboard" text
        if line.to_lowercase() == "sideboard" {
            continue;
        }
        // separate by first space to get number and name
        let words: Vec<&str> = line.split_whitespace().collect();
        if words.len() < 2 {
            continue; // NOTE: not a valid line
        }
        // convert number to integer
        let str_num = words[0].parse::<u64>()?;
        let card_name = words[1..].join(" ");
        decklist.push(CollectionCard {
            name: card_name,
            quantity: str_num,
        });
    }
    Ok(decklist)
}
