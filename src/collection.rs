use std::{
    error::Error,
    fmt::Display,
    fs::{self, File},
};

use csv::ReaderBuilder;
use diacritics::remove_diacritics;
use serde::Deserialize;

use crate::database::scryfall::{CardLayouts, ScryfallCard};

/// simple card format for collections and decklists
/// just the card name and the quantity
#[derive(Deserialize, Clone, Debug)]
pub struct CollectionCard {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Count")]
    pub quantity: u64,
}

impl Display for CollectionCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.quantity, self.name)
    }
}

/// reads in Moxfield collection CSV and turns it into a Vec<CollectionCard>
pub async fn read_moxfield_collection(
    file_name: String,
) -> Result<Vec<CollectionCard>, Box<dyn Error>> {
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
        if !collection_out.is_empty() {
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

/// reads in a decklist file in this format:
/// ## Card Name
/// safely skips over Sideboard label, blank lines, etc.
pub fn read_decklist(file_name: String) -> Result<Vec<CollectionCard>, Box<dyn Error>> {
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

/// compares the decklist to the loaded collection
/// outputs a list of missing cards
pub async fn find_missing_cards(
    collection: Vec<CollectionCard>,
    decklist: Vec<CollectionCard>,
) -> Option<Vec<CollectionCard>> {
    let mut missing_cards: Vec<CollectionCard> = Vec::new();
    for card in decklist.iter() {
        let mut found = false;
        for item in collection.iter() {
            if item.name == card.name {
                found = true;
                if item.quantity < card.quantity {
                    let mut missing_card = card.clone();
                    missing_card.quantity -= item.quantity;
                    missing_cards.push(missing_card);
                }
                break;
            }
        }
        if !found {
            missing_cards.push(card.clone());
        }
    }
    if missing_cards.is_empty() {
        None
    } else {
        Some(missing_cards)
    }
}

/// compares missing card to Scryfall database (if included)
/// if the card isn't found in the database, prompt the user to check the spelling
pub fn check_missing(database: &[ScryfallCard], missing_card: &CollectionCard) -> String {
    let mut found = false;
    for card in database.iter() {
        if remove_diacritics(&missing_card.name) == remove_diacritics(&card.name)
            || (remove_diacritics(&card.name)
                .find(&remove_diacritics(&missing_card.name))
                .is_some()
                && card.layout == CardLayouts::Transform)
        {
            found = true;
            break;
        }
    }
    if found {
        "".to_string()
    } else {
        " <------ This card was not found in database.  Check spelling?".to_string()
    }
}
