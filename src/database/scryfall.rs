/// structure for all Scryfall card data for a unique card
// TODO: map to JSON field names manually?  or rename?
pub struct ScryfallCard {
    pub object: ScryfallObject,
    pub id: String,
    pub oracle_id: String,
    pub multiverse_id: i64,
    pub mtgo_id: i64,
    pub mtgo_foil_id: i64,
    pub tcgplayer_id: i64,
    pub cardmarket_id: i64,
    pub name: String,
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
    pub type_line: Vec<CardTypes>,
    pub oracle_text: String,
    pub colors: MtGColors,
    pub color_identity: MtGColors,
    pub key_words: Vec<MtGKeyWords>,
    pub legalities: Legalities,
    pub games: Vec<GameFormat>,
    pub reserved: bool,
    pub foil: bool,
    pub nonfoil: bool,
    pub finishes: ScryfallFinishes,
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
    pub collector_number: i64,
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
pub enum ScryfallObject {
    Card,
    Token, // TODO: what types exist?
}

/// different languages for MTG printings
pub enum Languages {
    English,
    Spanish,
}

/// different card layout options
pub enum CardLayouts {
    Normal,
    Split,
}

/// scryfall image statuses
pub enum ImageStatus {
    HighRes,
    LowRes,
}

/// struct for all Scryfall image uris
pub struct ImageUris {
    pub small: String,
    pub normal: String,
    pub large: String,
    pub png: String,
    pub art_crop: String,
    pub border_crop: String,
}

/// MtG card types
pub enum CardTypes {
    Artifact,
    Creature,
    Instant,
    Sorcery,
    Enchantment,
    Land,
}

pub enum MtGColors {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colorless,
}

/// MtG card keywords
pub enum MtGKeyWords {
    Trample,
    Haste,
    FirstStrike,
} // TODO: the rest of these

/// card legality options for a specific format
pub enum Legality {
    Legal,
    NotLegal,
    Restricted,
}

/// contains legal status of card in every Scryfall format
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
pub enum GameFormat {
    Paper,
    MTGO,
}

/// different kinds of finishes recognized by Scryfall
pub enum ScryfallFinishes {
    Foil,
    NonFoil,
}

/// Scryfall set classifications
pub enum ScryfallSetType {
    Core,
    Expansion,
}

/// card rarities
pub enum MtGRarity {
    Common,
    Uncommon,
    Rare,
    Mythic,
}

/// card border colors
pub enum BorderColor {
    White,
    Black,
}

/// Scryfall prices struct
pub struct ScryfallPrices {
    pub usd: Option<f64>,
    pub usd_foil: Option<f64>,
    pub usd_etched: Option<f64>,
    pub euro: Option<f64>,
    pub euro_foil: Option<f64>,
    pub tix: Option<f64>,
}

/// struct of all of Scryfall's related URIs
pub struct ScryfallRelated {
    pub gatherer: String,
    pub tcgplayer_infinite_articles: String,
    pub tcgplayer_infinite_decks: String,
    pub edhrec: String,
}

/// struct of Scryfall purchase URIs
pub struct ScryfallPurchase {
    pub tcgplayer: String,
    pub cardmarket: String,
    pub cardhoarder: String,
}
