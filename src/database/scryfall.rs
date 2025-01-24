use std::{error::Error, ffi::OsString, fs};

use serde::Deserialize;

/// structure for all Scryfall card data for a unique card
// TODO: map to JSON field names manually?  or rename?
#[derive(Deserialize)]
pub struct ScryfallCard {
    pub object: ScryfallObject,
    pub id: String,
    pub oracle_id: String,
    pub multiverse_ids: Vec<i64>,
    pub mtgo_id: i64,
    pub mtgo_foil_id: i64,
    pub tcgplayer_id: i64,
    pub cardmarket_id: i64,
    pub name: String,
    #[serde(rename = "lang")]
    pub language: Languages,
    pub released_at: String, // TODO: some kind of date?
    pub uri: String,         // TODO: some kind of URL type?
    pub scryfall_uri: String,
    pub layout: CardLayouts,
    pub highres_image: bool,
    pub image_status: ImageStatus,
    pub image_uris: ImageUris,
    pub manacost: i64,
    pub cmc: f64,
    pub type_line: String, // Vec<CardTypes>,
    pub oracle_text: String,
    pub colors: Option<Vec<MtGColors>>,
    pub color_identity: Option<Vec<MtGColors>>,
    pub key_words: Vec<MtGKeyWords>,
    pub legalities: Legalities,
    pub games: Vec<GameFormat>,
    pub reserved: bool,
    pub foil: bool,
    pub nonfoil: bool,
    pub finishes: Vec<ScryfallFinishes>,
    pub oversized: bool,
    pub promo: bool,
    pub reprint: bool,
    pub variation: bool,
    pub set_id: String,
    pub set: String,
    pub set_name: String,
    pub set_type: ScryfallSetType,
    pub set_uri: String,
    pub set_search_uri: String,
    pub rulings_uri: String,
    pub prints_search_uri: String,
    pub collector_number: String, // TODO: convert to i64 in post?
    pub digital: bool,
    pub rarity: MtGRarity,
    pub flavor_text: String,
    pub card_back_id: String,
    pub artist: String,
    pub artist_ids: Vec<String>,
    pub illustration_id: String,
    pub border_color: BorderColor,
    pub frame: String, // TODO: year?
    pub full_art: bool,
    pub textless: bool,
    pub booster: bool,
    pub story_spotlight: bool,
    pub edhrec_rank: i64,
    pub prices: ScryfallPrices,
    pub related_uris: ScryfallRelated,
    pub purchase_uris: ScryfallPurchase,
}

/// represents different kinds of Scryfall objects
#[derive(Deserialize)]
pub enum ScryfallObject {
    #[serde(rename = "card")]
    Card,
    #[serde(rename = "token")]
    Token, // TODO: what types exist?
}

/// different languages for MTG printings
#[derive(Deserialize)]
pub enum Languages {
    #[serde(rename = "en")]
    English,
    Spanish,
}

/// different card layout options
#[derive(Deserialize)]
pub enum CardLayouts {
    #[serde(rename = "normal")]
    Normal,
    Split,
}

/// scryfall image statuses
#[derive(Deserialize)]
pub enum ImageStatus {
    #[serde(rename = "highres_scan")]
    HighRes,
    LowRes,
}

/// struct for all Scryfall image uris
#[derive(Deserialize)]
pub struct ImageUris {
    pub small: String,
    pub normal: String,
    pub large: String,
    pub png: String,
    pub art_crop: String,
    pub border_crop: String,
}

/// MtG card types
#[derive(Deserialize)]
pub enum CardTypes {
    Artifact,
    Creature,
    Instant,
    Sorcery,
    Enchantment,
    Land,
}

#[derive(Deserialize)]
pub enum MtGColors {
    White,
    Blue,
    Black,
    Red,
    Green,
}

/// MtG card keywords
#[derive(Deserialize)]
pub enum MtGKeyWords {
    Trample,
    Haste,
    FirstStrike,
} // TODO: the rest of these

/// card legality options for a specific format
#[derive(Deserialize)]
pub enum Legality {
    #[serde(rename = "legal")]
    Legal,
    #[serde(rename = "not_legal")]
    NotLegal,
    #[serde(rename = "restricted")]
    Restricted,
}

/// contains legal status of card in every Scryfall format
#[derive(Deserialize)]
pub struct Legalities {
    pub standard: Legality,
    pub future: Legality,
    pub historic: Legality,
    pub timeless: Legality,
    pub gladiator: Legality,
    pub pioneer: Legality,
    pub explorer: Legality,
    pub modern: Legality,
    pub legacy: Legality,
    pub pauper: Legality,
    pub vintage: Legality,
    pub penny: Legality,
    pub commander: Legality,
    pub oathbreaker: Legality,
    pub standardbrawl: Legality,
    pub brawl: Legality,
    pub alchemy: Legality,
    pub paupercommander: Legality,
    pub duel: Legality,
    pub oldschool: Legality,
    pub premodern: Legality,
    pub predh: Legality,
}

/// different game formats
#[derive(Deserialize)]
pub enum GameFormat {
    #[serde(rename = "paper")]
    Paper,
    #[serde(rename = "mtgo")]
    MTGO,
}

/// different kinds of finishes recognized by Scryfall
#[derive(Deserialize)]
pub enum ScryfallFinishes {
    #[serde(rename = "foil")]
    Foil,
    #[serde(rename = "nonfoil")]
    NonFoil,
}

/// Scryfall set classifications
#[derive(Deserialize)]
pub enum ScryfallSetType {
    #[serde(rename = "core")]
    Core,
    #[serde(rename = "expansion")]
    Expansion,
}

/// card rarities
#[derive(Deserialize)]
pub enum MtGRarity {
    #[serde(rename = "common")]
    Common,
    #[serde(rename = "uncommon")]
    Uncommon,
    #[serde(rename = "rare")]
    Rare,
    #[serde(rename = "mythic")]
    Mythic,
}

/// card border colors
#[derive(Deserialize)]
pub enum BorderColor {
    #[serde(rename = "white")]
    White,
    #[serde(rename = "black")]
    Black,
}

/// Scryfall prices struct
#[derive(Deserialize)]
pub struct ScryfallPrices {
    pub usd: Option<String>,        // Option<f64>,
    pub usd_foil: Option<String>,   // Option<f64>,
    pub usd_etched: Option<String>, // Option<f64>,
    pub euro: Option<String>,       // Option<f64>,
    pub euro_foil: Option<String>,  // Option<f64>,
    pub tix: Option<String>,        // Option<f64>,
}

/// struct of all of Scryfall's related URIs
#[derive(Deserialize)]
pub struct ScryfallRelated {
    pub gatherer: String,
    pub tcgplayer_infinite_articles: String,
    pub tcgplayer_infinite_decks: String,
    pub edhrec: String,
}

/// struct of Scryfall purchase URIs
#[derive(Deserialize)]
pub struct ScryfallPurchase {
    pub tcgplayer: String,
    pub cardmarket: String,
    pub cardhoarder: String,
}

/// reads provided JSON database file and produces a vector of ScryfallCard objects
pub fn read_scryfall_database(path: &OsString) -> Result<Vec<ScryfallCard>, Box<dyn Error>> {
    let file_text = fs::read_to_string(path)?;
    let test: Result<Vec<ScryfallCard>, serde_json::Error> = serde_json::from_str(&file_text);
    Ok(test?)
}
