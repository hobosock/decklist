use std::{
    error::Error,
    fmt::Display,
    fs::{self, File},
};

use csv::ReaderBuilder;
use diacritics::remove_diacritics;
use serde::Deserialize;

use crate::database::scryfall::{match_card, CardLayouts, Legality, ScryfallCard};

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
            if remove_diacritics(&item.name) == remove_diacritics(&card.name) {
                found = true;
                if item.quantity < card.quantity {
                    let mut missing_card = card.clone();
                    missing_card.quantity -= item.quantity;
                    missing_cards.push(missing_card);
                }
                break;
            }
            // NOTE: split/transofrm/etc cards are tricky to match
            if item.name.contains("//") {
                let half_name = item.name.split("//").collect::<Vec<&str>>()[0].trim();
                if remove_diacritics(half_name) == remove_diacritics(&card.name) {
                    found = true;
                    if item.quantity < card.quantity {
                        let mut missing_card = card.clone();
                        missing_card.quantity -= item.quantity;
                        missing_cards.push(missing_card);
                    }
                    break;
                }
            }
            if card.name.contains("//") {
                let half_name = card.name.split("//").collect::<Vec<&str>>()[0].trim();
                if remove_diacritics(half_name) == remove_diacritics(&item.name) {
                    found = true;
                    if item.quantity < card.quantity {
                        let mut missing_card = card.clone();
                        missing_card.quantity -= item.quantity;
                        missing_cards.push(missing_card);
                    }
                    break;
                }
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
        // NOTE: dual/split/transform card names are tricky - match on a partial
        let dual = if card.layout == CardLayouts::Transform
            || card.layout == CardLayouts::Flip
            || card.layout == CardLayouts::Split
            || card.layout == CardLayouts::ModalDualFaceCard
        {
            true
        } else {
            false
        };
        if remove_diacritics(&missing_card.name) == remove_diacritics(&card.name)
            || (remove_diacritics(&card.name)
                .find(&remove_diacritics(&missing_card.name))
                .is_some()
                && dual)
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

/// simpler format legality struct, just a basic true/false
/// default to true, then eliminate formats as you go through cards
pub struct FormatLegal {
    pub standard: bool,
    pub future: bool,
    pub historic: bool,
    pub timeless: bool,
    pub gladiator: bool,
    pub pioneer: bool,
    pub explorer: bool,
    pub modern: bool,
    pub legacy: bool,
    pub pauper: bool,
    pub vintage: bool,
    pub penny: bool,
    pub commander: bool,
    pub oathbreaker: bool,
    pub standardbrawl: bool,
    pub brawl: bool,
    pub alchemy: bool,
    pub paupercommander: bool,
    pub duel: bool,
    pub oldschool: bool,
    pub premodern: bool,
    pub predh: bool,
}

impl Default for FormatLegal {
    fn default() -> Self {
        Self {
            standard: true,
            future: true,
            historic: true,
            timeless: true,
            gladiator: true,
            pioneer: true,
            explorer: true,
            modern: true,
            legacy: true,
            pauper: true,
            vintage: true,
            penny: true,
            commander: true,
            oathbreaker: true,
            standardbrawl: true,
            brawl: true,
            alchemy: true,
            paupercommander: true,
            duel: true,
            oldschool: true,
            premodern: true,
            predh: true,
        }
    }
}

/// convert multiple types of Scryfall Legalities into a simple true/false
/// TODO: deal with Restricted type somehow?
fn convert_legal(legal: Legality) -> bool {
    match legal {
        Legality::Legal => true,
        Legality::Restricted => true,
        _ => false,
    }
}

/// checks decklist for legality, and outputs a structure with the results
pub async fn check_legality(decklist: &[CollectionCard], database: &[ScryfallCard]) -> FormatLegal {
    // TODO: check number of cards?
    let mut legal = FormatLegal::default();
    for card in decklist {
        if let Some(matched) = match_card(&card.name, database) {
            // go through every format - if still true, check current card legality
            if legal.standard {
                legal.standard = convert_legal(matched.legalities.standard); // only go to false
            }
            if legal.future {
                legal.future = convert_legal(matched.legalities.future);
            }
            if legal.historic {
                legal.historic = convert_legal(matched.legalities.historic);
            }
            if legal.timeless {
                legal.timeless = convert_legal(matched.legalities.timeless);
            }
            if legal.gladiator {
                legal.gladiator = convert_legal(matched.legalities.gladiator);
            }
            if legal.pioneer {
                legal.pioneer = convert_legal(matched.legalities.pioneer);
            }
            if legal.explorer {
                legal.explorer = convert_legal(matched.legalities.explorer);
            }
            if legal.modern {
                legal.modern = convert_legal(matched.legalities.modern);
            }
            if legal.legacy {
                legal.legacy = convert_legal(matched.legalities.legacy);
            }
            if legal.pauper {
                legal.pauper = convert_legal(matched.legalities.pauper);
            }
            if legal.vintage {
                legal.vintage = convert_legal(matched.legalities.vintage);
            }
            if legal.penny {
                legal.penny = convert_legal(matched.legalities.penny);
            }
            if legal.commander {
                legal.commander = convert_legal(matched.legalities.commander);
            }
            if legal.oathbreaker {
                legal.oathbreaker = convert_legal(matched.legalities.oathbreaker);
            }
            if legal.standardbrawl {
                legal.standardbrawl = convert_legal(matched.legalities.standardbrawl);
            }
            if legal.brawl {
                legal.brawl = convert_legal(matched.legalities.brawl);
            }
            if legal.alchemy {
                legal.alchemy = convert_legal(matched.legalities.alchemy);
            }
            if legal.paupercommander {
                legal.paupercommander = convert_legal(matched.legalities.paupercommander);
            }
            if legal.duel {
                legal.duel = convert_legal(matched.legalities.duel);
            }
            if legal.oldschool {
                legal.oldschool = convert_legal(matched.legalities.oldschool);
            }
            if legal.premodern {
                legal.premodern = convert_legal(matched.legalities.premodern);
            }
            if legal.predh {
                legal.predh = convert_legal(matched.legalities.predh);
            }
        }
    }
    legal
}
