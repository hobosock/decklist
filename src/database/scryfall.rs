use std::{error::Error, fs, path::PathBuf};

use diacritics::remove_diacritics;
use serde::Deserialize;

/// structure for all Scryfall card data for a unique card
// TODO: map to JSON field names manually?  or rename?
#[derive(Deserialize, Clone)]
pub struct ScryfallCard {
    pub object: ScryfallObject,
    pub id: String,
    pub oracle_id: String,
    pub multiverse_ids: Vec<i64>,
    #[serde(default)]
    pub mtgo_id: i64,
    #[serde(default)] // NOTE: not all cards have this field
    pub mtgo_foil_id: i64,
    #[serde(default)]
    pub tcgplayer_id: i64,
    #[serde(default)]
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
    #[serde(default)]
    pub image_uris: ImageUris,
    #[serde(default)]
    pub mana_cost: String, // i64,
    pub cmc: f64,
    pub type_line: String, // Vec<CardTypes>,
    #[serde(default)]
    pub oracle_text: String,
    pub colors: Option<Vec<MtGColors>>,
    pub color_identity: Option<Vec<MtGColors>>,
    pub keywords: Vec<String>, // Vec<MtGKeyWords>,
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
    #[serde(default)]
    pub rulings_uri: String,
    pub prints_search_uri: String,
    pub collector_number: String, // TODO: convert to i64 in post?
    pub digital: bool,
    pub rarity: MtGRarity,
    #[serde(default)]
    pub flavor_text: String,
    #[serde(default)]
    pub card_back_id: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)] // TODO: replace with function call instead?  see serde docs
    pub artist_ids: Vec<String>,
    #[serde(default)]
    pub illustration_id: String,
    pub border_color: BorderColor,
    pub frame: String, // TODO: year?
    pub full_art: bool,
    pub textless: bool,
    pub booster: bool,
    pub story_spotlight: bool,
    #[serde(default)]
    pub edhrec_rank: i64,
    pub prices: ScryfallPrices,
    pub related_uris: ScryfallRelated,
    #[serde(default)]
    pub purchase_uris: ScryfallPurchase,
}

impl ScryfallCard {
    pub fn price_to_string(self, quantity: u64, price_type: PriceType) -> String {
        let currency_str = match price_type {
            PriceType::USD => "$".to_string(),
            PriceType::Euro => "€".to_string(),
            PriceType::Tix => "Tix ".to_string(),
        };
        if let Some(price) = match price_type {
            PriceType::USD => self.prices.usd,
            PriceType::Euro => self.prices.euro,
            PriceType::Tix => self.prices.tix,
        } {
            match price.parse::<f64>() {
                Ok(price_num) => {
                    format!(
                        "[{}] x{} = {}{}",
                        price_num,
                        quantity,
                        currency_str,
                        price_num * quantity as f64
                    )
                }
                Err(_e) => price,
            }
        } else {
            "".to_string()
        }
    }
}

/// represents different kinds of Scryfall objects
#[derive(Deserialize, Clone)]
pub enum ScryfallObject {
    #[serde(rename = "card")]
    Card,
    #[serde(rename = "token")]
    Token, // TODO: what types exist?
}

/// different languages for MTG printings
#[derive(Deserialize, Clone)]
pub enum Languages {
    #[serde(rename = "en")]
    English,
    #[serde(rename = "ja")]
    Japanese,
}

/// different card layout options
#[derive(Deserialize, Clone, PartialEq)]
pub enum CardLayouts {
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "art_series")]
    ArtSeries,
    #[serde(rename = "token")]
    Token,
    #[serde(rename = "class")]
    Class,
    #[serde(rename = "planar")]
    Planar,
    #[serde(rename = "saga")]
    Saga,
    #[serde(rename = "scheme")]
    Scheme,
    #[serde(rename = "double_faced_token")]
    DoubleFacedToken,
    #[serde(rename = "meld")]
    Meld,
    #[serde(rename = "prototype")]
    Prototype,
    #[serde(rename = "vanguard")]
    Vanguard,
    #[serde(rename = "transform")]
    Transform,
    #[serde(rename = "emblem")]
    Emblem,
    #[serde(rename = "modal_dfc")]
    ModalDualFaceCard,
    #[serde(rename = "split")]
    Split,
    #[serde(rename = "adventure")]
    Adventure,
    #[serde(rename = "augment")]
    Augment,
    #[serde(rename = "flip")]
    Flip,
    #[serde(rename = "host")]
    Host,
    #[serde(rename = "mutate")]
    Mutate,
    #[serde(rename = "leveler")]
    Leveler,
    #[serde(rename = "case")]
    Case,
}

/// scryfall image statuses
#[derive(Deserialize, Clone)]
pub enum ImageStatus {
    #[serde(rename = "highres_scan")]
    HighRes,
    #[serde(rename = "lowres")]
    LowRes,
    #[serde(rename = "missing")]
    Missing,
}

/// struct for all Scryfall image uris
#[derive(Deserialize, Default, Clone)]
pub struct ImageUris {
    pub small: String,
    pub normal: String,
    pub large: String,
    pub png: String,
    pub art_crop: String,
    pub border_crop: String,
}

/// MtG card types
#[derive(Deserialize, Clone)]
pub enum CardTypes {
    Artifact,
    Creature,
    Instant,
    Sorcery,
    Enchantment,
    Land,
}

#[derive(Deserialize, Clone)]
pub enum MtGColors {
    #[serde(rename = "W")]
    White,
    #[serde(rename = "U")]
    Blue,
    #[serde(rename = "B")]
    Black,
    #[serde(rename = "R")]
    Red,
    #[serde(rename = "G")]
    Green,
}

/// MtG card keywords
#[derive(Deserialize, Clone)]
pub enum MtGKeyWords {
    Trample,
    Haste,
    #[serde(rename = "First strike")]
    FirstStrike,
    Enchant,
    Entwine,
    Flying,
    Food,
    Adapt,
    Treasure,
    Affinity,
    Horsemanship,
    Deathtouch,
    Defender,
    #[serde(rename = "Double strike")]
    DoubleStrike,
    Equip,
    Flash,
    Hexproof,
    Indestructible,
    Lifelink,
    Protection,
    Reach,
    Vigilance,
    Ward,
    Menace,
    Activate,
    Attach,
    Cast,
    Counter,
    Create,
    Destroy,
    Discard,
    Exchange,
    Exile,
    Fight,
    Mill,
    Play,
    Reveal,
    Sacrifice,
    Scry,
    Search,
    Shuffle,
    Tap,
    Untap,
    Intimidate,
    Landwalk,
    Shroud,
    Banding,
    Rampage,
    #[serde(rename = "Cumulative upkeep")]
    CumulativeUpkeep,
    Flanking,
    Phasing,
    Buyback,
    Shadow,
    Cycling,
    Echo,
    Fading,
    Kicker,
    Flashback,
    Madness,
    Fear,
    Morph,
    Amplify,
    Provoke,
    Storm,
    Modular,
    Sunburst,
    Bushido,
    Soulshift,
    Splice,
    Offering,
    Ninjutsu,
    Epic,
    Convoke,
    Dredge,
    Transmute,
    Bloodthirst,
    Haunt,
    Replicate,
    Forecast,
    Graft,
    Recover,
    Ripple,
    #[serde(rename = "Split second")]
    SplitSecond,
    Vanishing,
    Absorb,
    AuraSwap,
    Delve,
    Fortify,
    Frenzy,
    Gravestorm,
    Poisonous,
    Transfigure,
    Champion,
    Changeling,
    Evoke,
    Hideaway,
    Prowl,
    Reinforce,
    Conspire,
    Connive,
    Persist,
    Wither,
    Retrace,
    Devour,
    Exalted,
    Unearth,
    Cascade,
    Annihilator,
    LevelUp,
    Rebound,
    #[serde(rename = "Totem armor")]
    TotemArmor,
    Infect,
    #[serde(rename = "Battle cry")]
    BattleCry,
    #[serde(rename = "Living weapon")]
    LivingWeapon,
    Undying,
    Miracle,
    Soulbond,
    Overload,
    Scavenge,
    Unleash,
    Cipher,
    Evolve,
    Extort,
    Fuse,
    Bestow,
    Tribute,
    Dethrone,
    #[serde(rename = "Hidden agenda")]
    HiddenAgenda,
    Outlast,
    Prowess,
    Dash,
    Exploit,
    Renown,
    Awaken,
    Devoid,
    Ingest,
    Myriad,
    Surge,
    Skulk,
    Emerge,
    Escalate,
    Melee,
    Crew,
    Fabricate,
    Partner,
    Undaunted,
    Improvise,
    Aftermath,
    Embalm,
    Eternalize,
    Afflict,
    Ascend,
    Assist,
    JumpStart,
    Mentor,
    Afterlife,
    Riot,
    Spectacle,
    Escape,
    Companion,
    Mutate,
    Encore,
    Boast,
    Foretell,
    Demonstrate,
    Daybound,
    Nightbound,
    Disturb,
    Decayed,
    Cleave,
    Training,
    Completed,
    Reconfigure,
    Fateseal,
    Clash,
    Planeswalk,
    SetInMotion,
    Abandon,
    Proliferate,
    Transform,
    Detain,
    Populate,
    Monstrosity,
    Vote,
    Bolster,
    Manifest,
    Support,
    Investigate,
    Meld,
    Goad,
    Exert,
    Explore,
    Assemble,
    Surveil,
    Amass,
    Learn,
    Venture,
    Swampwalk,
    Islandwalk,
    Plainswalk,
    Forestwalk,
    Mountainwalk,
    Enrage,
    Domain,
    Corrupted,
    #[serde(rename = "Role token")]
    RoleToken,
    Revolt,
    #[serde(rename = "Join forces")]
    JoinForces,
    Plot,
    Multikicker,
    Spree,
    #[serde(rename = "Shrieking Gargoyles")]
    ShriekingGargoyles,
    Heroic,
    Discover,
    Cohort,
    Eerie,
    #[serde(rename = "Open an Attraction")]
    OpenAnAttraction,
    #[serde(rename = "Phalanx Commander")]
    PhalanxCommander,
    #[serde(rename = "Choose a background")]
    ChooseABackground,
    Addendum,
    Megamorph,
    Morbid,
    Compleated,
    Threshold,
    Prototype,
    Seek,
    Conjure,
    Magecraft,
    Rulebreaker,
    Feed,
    Friends,
    Descend,
    Channel,
    Gift,
    #[serde(rename = "Partner with")]
    ParternWith,
    #[serde(rename = "Will of the council")]
    WillOfTheCouncil,
    Specialize,
    Chroma,
    Suspend,
    Plainscycling,
    Islandcycling,
    Swampcycling,
    Forestcycling,
    Mountaincycling,
    Landcycling,
    Typecycling,
    #[serde(rename = "Collect evidence")]
    CollectEvidence,
    #[serde(rename = "Basic landcycling")]
    BasicLandcycling,
    Landfall,
    Disguise,
    Incubate,
    #[serde(rename = "Pack tactics")]
    PackTactics,
    Augment,
    Coven,
    Bloodrush,
    #[serde(rename = "Secret council")]
    SecretCouncil,
    Blitz,
    #[serde(rename = "Manifest dread")]
    ManifestDread,
    #[serde(rename = "Hexproof from")]
    HexproofFrom,
    #[serde(rename = "Crash Landing")]
    CrashLanding,
    #[serde(rename = "Medicus Ministorum")]
    MedicusMinistorum,
    #[serde(rename = "Tempting offer")]
    TemptingOffer,
    Metalcraft,
    #[serde(rename = "Sonic Booster")]
    SonicBooster,
    #[serde(rename = "Doctor's companion")]
    DoctorsCompanion,
    Bargain,
    #[serde(rename = "Double agenda")]
    DoubleAenda,
    Toxic,
    Casualty,
    #[serde(rename = "Venture into the dungeon")]
    Dungeon,
    Delirium,
    #[serde(rename = "Spell mastery")]
    SpellMastery,
    #[serde(rename = "Friends forever")]
    FriendsForever,
    Kinship,
    Disarm,
    Detonate,
    Avoidance,
    Grandeur,
}

/// card legality options for a specific format
#[derive(Deserialize, Clone)]
pub enum Legality {
    #[serde(rename = "legal")]
    Legal,
    #[serde(rename = "not_legal")]
    NotLegal,
    #[serde(rename = "restricted")]
    Restricted,
    #[serde(rename = "banned")]
    Banned,
}

/// contains legal status of card in every Scryfall format
#[derive(Deserialize, Clone)]
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

impl Default for Legalities {
    fn default() -> Self {
        Self {
            standard: Legality::NotLegal,
            future: Legality::NotLegal,
            historic: Legality::NotLegal,
            timeless: Legality::NotLegal,
            gladiator: Legality::NotLegal,
            pioneer: Legality::NotLegal,
            explorer: Legality::NotLegal,
            modern: Legality::NotLegal,
            legacy: Legality::NotLegal,
            pauper: Legality::NotLegal,
            vintage: Legality::NotLegal,
            penny: Legality::NotLegal,
            commander: Legality::NotLegal,
            oathbreaker: Legality::NotLegal,
            standardbrawl: Legality::NotLegal,
            brawl: Legality::NotLegal,
            alchemy: Legality::NotLegal,
            paupercommander: Legality::NotLegal,
            duel: Legality::NotLegal,
            oldschool: Legality::NotLegal,
            premodern: Legality::NotLegal,
            predh: Legality::NotLegal,
        }
    }
}

/// different game formats
#[derive(Deserialize, Clone)]
pub enum GameFormat {
    #[serde(rename = "paper")]
    Paper,
    #[serde(rename = "mtgo")]
    MTGO,
    #[serde(rename = "arena")]
    Arena,
    #[serde(rename = "astral")]
    Astral,
    #[serde(rename = "sega")]
    Sega,
}

/// different kinds of finishes recognized by Scryfall
#[derive(Deserialize, Clone)]
pub enum ScryfallFinishes {
    #[serde(rename = "foil")]
    Foil,
    #[serde(rename = "nonfoil")]
    NonFoil,
    #[serde(rename = "etched")]
    Etched,
}

/// Scryfall set classifications
#[derive(Deserialize, Clone)]
pub enum ScryfallSetType {
    #[serde(rename = "core")]
    Core,
    #[serde(rename = "commander")]
    Commander,
    #[serde(rename = "alchemy")]
    Alchemy,
    #[serde(rename = "expansion")]
    Expansion,
    #[serde(rename = "masters")]
    Masters,
    #[serde(rename = "draft_innovation")]
    DraftInnovation,
    #[serde(rename = "funny")]
    Funny,
    #[serde(rename = "memorabilia")]
    Memorabilia,
    #[serde(rename = "token")]
    Token,
    #[serde(rename = "duel_deck")]
    DuelDeck,
    #[serde(rename = "starter")]
    Starter,
    #[serde(rename = "planechase")]
    Planechase,
    #[serde(rename = "archenemy")]
    ArchEnemy,
    #[serde(rename = "minigame")]
    Minigame,
    #[serde(rename = "box")]
    Box,
    #[serde(rename = "vanguard")]
    Vanguard,
    #[serde(rename = "promo")]
    Promo,
    #[serde(rename = "masterpiece")]
    Masterpiece,
    #[serde(rename = "arsenal")]
    Arsenal,
    #[serde(rename = "treasure_chest")]
    TreasureChest,
}

/// card rarities
#[derive(Deserialize, Clone)]
pub enum MtGRarity {
    #[serde(rename = "common")]
    Common,
    #[serde(rename = "uncommon")]
    Uncommon,
    #[serde(rename = "rare")]
    Rare,
    #[serde(rename = "mythic")]
    Mythic,
    #[serde(rename = "special")]
    Special,
    #[serde(rename = "bonus")]
    Bonus,
}

/// card border colors
#[derive(Deserialize, Clone)]
pub enum BorderColor {
    #[serde(rename = "white")]
    White,
    #[serde(rename = "black")]
    Black,
    #[serde(rename = "silver")]
    Silver,
    #[serde(rename = "borderless")]
    Borderless,
    #[serde(rename = "gold")]
    Gold,
}

/// Scryfall prices struct
#[derive(Deserialize, Clone)]
pub struct ScryfallPrices {
    pub usd: Option<String>,        // Option<f64>,
    pub usd_foil: Option<String>,   // Option<f64>,
    pub usd_etched: Option<String>, // Option<f64>,
    pub euro: Option<String>,       // Option<f64>,
    pub euro_foil: Option<String>,  // Option<f64>,
    pub tix: Option<String>,        // Option<f64>,
}

/// selected currency to show prices in
pub enum PriceType {
    USD,
    Euro,
    Tix,
}

/// struct of all of Scryfall's related URIs
#[derive(Deserialize, Clone)]
pub struct ScryfallRelated {
    #[serde(default)]
    pub gatherer: String,
    #[serde(default)]
    pub tcgplayer_infinite_articles: String,
    #[serde(default)]
    pub tcgplayer_infinite_decks: String,
    #[serde(default)]
    pub edhrec: String,
}

/// struct of Scryfall purchase URIs
#[derive(Deserialize, Default, Clone)]
pub struct ScryfallPurchase {
    #[serde(default)]
    pub tcgplayer: String,
    #[serde(default)]
    pub cardmarket: String,
    #[serde(default)]
    pub cardhoarder: String,
}

/// reads provided JSON database file and produces a vector of ScryfallCard objects
pub fn read_scryfall_database(path: &PathBuf) -> Result<Vec<ScryfallCard>, Box<dyn Error>> {
    let file_text = fs::read_to_string(path)?;
    let test: Result<Vec<ScryfallCard>, serde_json::Error> = serde_json::from_str(&file_text);
    Ok(test?)
}

// TODO: maybe replace the manual implementations of this elsewhere?
/// takes card name and finds matching card in database
pub fn match_card(cardname: &str, database: &[ScryfallCard]) -> Option<ScryfallCard> {
    let mut found = None;
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
        if remove_diacritics(cardname) == remove_diacritics(&card.name)
            || (remove_diacritics(&card.name)
                .find(&remove_diacritics(cardname))
                .is_some()
                && dual)
        {
            found = Some(card.clone());
            break;
        }
    }
    found
}
