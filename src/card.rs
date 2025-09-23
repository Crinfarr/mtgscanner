use std::f32::consts::PI;

use serde::{Deserialize, de::Unexpected};

use crate::card;

#[derive(Deserialize)]
#[allow(unused)]
pub struct Card {
    //core fields
    pub arena_id: Option<String>,
    pub id: String,
    pub lang: CardLang,
    pub mtgo_id: Option<u32>,
    pub mtgo_foil_id: Option<u32>,
    pub multiverse_ids: Option<Vec<u32>>,
    pub tcgplayer_id: Option<u32>,
    pub tcgplayer_etched_id: Option<u32>,
    pub cardmarket_id: Option<u32>,
    pub object: String,
    pub layout: CardLayout,
    pub oracle_id: Option<String>,
    pub prints_search_uri: String,
    pub rulings_uri: String,
    pub scryfall_uri: String,
    pub uri: String,
    //gameplay fields
    pub all_parts: Option<Vec<RelatedObject>>,
    pub card_faces: Option<Vec<CardFace>>,
    pub cmc: f64,
    pub color_identity: Vec<CardColor>,
    pub color_indicator: Option<Vec<CardColor>>,
    pub colors: Option<Vec<CardColor>>,
    pub defense: Option<String>,
    pub edhrec_rank: Option<i32>,
    pub game_changer: Option<bool>,
    pub hand_modifier: Option<String>,
    pub keywords: Vec<String>,
    pub legalities: CardLegalities,
    pub life_modifier: Option<String>,
    pub loyalty: Option<String>,
    pub mana_cost: Option<String>,
    pub name: String,
    pub oracle_text: Option<String>,
    pub penny_rank: Option<i32>,
    pub power: Option<String>,
    pub produced_mana: Option<Vec<CardColor>>,
    pub reserved: bool,
    pub toughness: Option<String>,
    pub type_line: String,
    //Print fields
    pub artist: Option<String>,
    pub artist_ids: Option<Vec<String>>,
    pub attraction_lights: Option<Vec<i8>>,
    pub booster: bool,
    pub border_color: CardBorderColor,
    pub card_back_id: String,
    pub collector_number: String,
    pub content_warning: Option<bool>,
    pub digital: bool,
    pub finishes: Vec<CardFinishes>,
    pub flavor_name: Option<String>,
    pub flavor_text: Option<String>,
    pub frame_effects: Option<Vec<CardFrameEffect>>,
    //TODO FRAMES +
    //https://scryfall.com/docs/api/cards
}
#[derive(Deserialize)]
#[allow(unused)]
pub struct CardLegalities {
    pub standard: LegalityState,
    pub future: LegalityState,
    pub historic: LegalityState,
    pub timeless: LegalityState,
    pub gladiator: LegalityState,
    pub pioneer: LegalityState,
    pub modern: LegalityState,
    pub legacy: LegalityState,
    pub pauper: LegalityState,
    pub vintage: LegalityState,
    pub penny: LegalityState,
    pub commander: LegalityState,
    pub oathbreaker: LegalityState,
    pub standardbrawl: LegalityState,
    pub brawl: LegalityState,
    pub alchemy: LegalityState,
    pub paupercommander: LegalityState,
    pub duel: LegalityState,
    pub oldschool: LegalityState,
    pub premodern: LegalityState,
    pub predh: LegalityState,
}

#[derive(Deserialize)]
#[allow(unused)]
pub struct CardFace {
    pub artist: Option<String>,
    pub artist_id: Option<String>,
    pub cmc: Option<f64>,
    pub color_indicator: Option<Vec<CardColor>>,
    pub colors: Option<Vec<CardColor>>,
    pub defense: Option<String>,
    pub flavor_text: Option<String>,
    pub illustration_id: Option<String>,
    pub image_uris: Option<String>,
    pub layout: Option<CardLayout>,
    pub mana_cost: String,
    pub name: String,
    pub object: String,
    pub oracle_id: Option<String>,
    pub oracle_text: Option<String>,
    pub power: Option<String>,
    pub printed_name: Option<String>,
    pub printed_text: Option<String>,
    pub printed_type_line: Option<String>,
    pub toughness: Option<String>,
    pub type_line: Option<String>,
    pub watermark: Option<String>,
}

#[derive(Deserialize)]
#[allow(unused)]
pub struct RelatedObject {
    pub id: String,
    pub object: String,
    pub component: String,
    pub name: String,
    pub type_line: String,
    pub uri: String,
}

pub enum CardFrameEffect {
    Legendary,
    Miracle,
    Enchantment,
    Draft,
    Devoid,
    Tombstone,
    Colorshifted,
    Inverted,
    SunMoonDFC,
    CompassLandDFC,
    OriginPwDFC,
    MoonEldraziDFC,
    WaxingAndWaningMoonDFC,
    Showcase,
    ExtendedArt,
    Companion,
    Etched,
    Snow,
    Lesson,
    ShatteredGlass,
    ConvertDFC,
    FanDFC,
    UpsideDownDFC,
    Spree,
}
impl<'de> Deserialize<'de> for CardFrameEffect {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let frame_effect = String::deserialize(deserializer)?;
        match frame_effect.as_str() {
            "legendary" => Ok(Self::Legendary),
            "miracle" => Ok(Self::Miracle),
            "enchantment" => Ok(Self::Enchantment),
            "draft" => Ok(Self::Draft),
            "devoid" => Ok(Self::Devoid),
            "tombstone" => Ok(Self::Tombstone),
            "colorshifted" => Ok(Self::Colorshifted),
            "inverted" => Ok(Self::Inverted),
            "sunmoondfc" => Ok(Self::SunMoonDFC),
            "compasslanddfc" => Ok(Self::CompassLandDFC),
            "originpwdfc" => Ok(Self::OriginPwDFC),
            "mooneldrazidfc" => Ok(Self::MoonEldraziDFC),
            "waxingandwaningmoondfc" => Ok(Self::WaxingAndWaningMoonDFC),
            "showcase" => Ok(Self::Showcase),
            "extendedart" => Ok(Self::ExtendedArt),
            "companion" => Ok(Self::Companion),
            "etched" => Ok(Self::Etched),
            "snow" => Ok(Self::Snow),
            "lesson" => Ok(Self::Lesson),
            "shatteredglass" => Ok(Self::ShatteredGlass),
            "convertdfc" => Ok(Self::ConvertDFC),
            "fandfc" => Ok(Self::FanDFC),
            "upsidedowndfc" => Ok(Self::UpsideDownDFC),
            "spree" => Ok(Self::Spree),
            other => Err(serde::de::Error::invalid_value(Unexpected::Str(other), &"any valid frame effect name"))
        }
    }
}

pub enum CardFinishes {
    Foil,
    Nonfoil,
    Etched,
}
impl<'de> Deserialize<'de> for CardFinishes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let card_finish = String::deserialize(deserializer)?;
        match card_finish.as_str() {
            "foil" => Ok(Self::Foil),
            "nonfoil" => Ok(Self::Nonfoil),
            "etched" => Ok(Self::Etched),
            other => Err(serde::de::Error::invalid_value(
                Unexpected::Str(other),
                &"Foil, Nonfoil, or Etched",
            )),
        }
    }
}

pub enum CardBorderColor {
    Black,
    White,
    Borderless,
    Yellow,
    Silver,
    Gold,
}
impl<'de> Deserialize<'de> for CardBorderColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let border_color = String::deserialize(deserializer)?;
        match border_color.as_str() {
            "black" => Ok(Self::Black),
            "white" => Ok(Self::White),
            "borderless" => Ok(Self::Borderless),
            "yellow" => Ok(Self::Yellow),
            "silver" => Ok(Self::Silver),
            "gold" => Ok(Self::Gold),
            other => Err(serde::de::Error::invalid_value(
                Unexpected::Str(other),
                &"Any valid border color",
            )),
        }
    }
}
pub enum LegalityState {
    Legal,
    NotLegal,
    Banned,
    Restricted,
}
impl<'de> Deserialize<'de> for LegalityState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let legal_state = String::deserialize(deserializer)?;
        match legal_state.as_str() {
            "legal" => Ok(Self::Legal),
            "not_legal" => Ok(Self::NotLegal),
            "banned" => Ok(Self::Banned),
            "restricted" => Ok(Self::Restricted),
            other => Err(serde::de::Error::invalid_value(
                Unexpected::Str(other),
                &"any valid legality",
            )),
        }
    }
}

pub enum CardColor {
    W,
    U,
    B,
    R,
    G,
}
impl<'de> Deserialize<'de> for CardColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let color_name = String::deserialize(deserializer)?;
        match color_name.as_str() {
            "W" => Ok(Self::W),
            "U" => Ok(Self::U),
            "B" => Ok(Self::B),
            "R" => Ok(Self::R),
            "G" => Ok(Self::G),
            other => Err(serde::de::Error::invalid_value(
                Unexpected::Str(other),
                &"W, U, B, R, or G",
            )),
        }
    }
}
pub enum CardLang {
    En,
    Es,
    Fr,
    De,
    It,
    Pt,
    Ja,
    Ko,
    Ru,
    Zhs,
    Zht,
    He,
    La,
    Grc,
    Ar,
    Sa,
    Ph,
    Qya,
}
impl<'de> Deserialize<'de> for CardLang {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let lang_name = String::deserialize(deserializer)?;
        match lang_name.as_str() {
            "en" => Ok(Self::En),
            "es" => Ok(Self::Es),
            "fr" => Ok(Self::Fr),
            "de" => Ok(Self::De),
            "it" => Ok(Self::It),
            "pt" => Ok(Self::Pt),
            "ja" => Ok(Self::Ja),
            "ko" => Ok(Self::Ko),
            "ru" => Ok(Self::Ru),
            "zhs" => Ok(Self::Zhs),
            "zht" => Ok(Self::Zht),
            "he" => Ok(Self::He),
            "la" => Ok(Self::La),
            "grc" => Ok(Self::Grc),
            "ar" => Ok(Self::Ar),
            "sa" => Ok(Self::Sa),
            "ph" => Ok(Self::Ph),
            "qya" => Ok(Self::Qya),
            other => Err(serde::de::Error::invalid_value(
                Unexpected::Str(other),
                &"Any valid lang",
            )),
        }
    }
}

pub enum CardLayout {
    Normal,
    Split,
    Flip,
    Transform,
    ModalDFC,
    Meld,
    Leveler,
    Class,
    Case,
    Saga,
    Adventure,
    Mutate,
    Prototype,
    Battle,
    Planar,
    Scheme,
    Vanguard,
    Token,
    DoubleFacedToken,
    Emblem,
    Augment,
    Host,
    ArtSeries,
    ReversibleCard,
}
impl<'de> Deserialize<'de> for CardLayout {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let variant = String::deserialize(deserializer)?;
        match variant.as_str() {
            "normal" => Ok(Self::Normal),
            "split" => Ok(Self::Split),
            "flip" => Ok(Self::Flip),
            "transform" => Ok(Self::Transform),
            "modal_dfc" => Ok(Self::ModalDFC),
            "meld" => Ok(Self::Meld),
            "leveler" => Ok(Self::Leveler),
            "class" => Ok(Self::Class),
            "case" => Ok(Self::Case),
            "saga" => Ok(Self::Saga),
            "adventure" => Ok(Self::Adventure),
            "mutate" => Ok(Self::Mutate),
            "prototype" => Ok(Self::Prototype),
            "battle" => Ok(Self::Battle),
            "planar" => Ok(Self::Planar),
            "scheme" => Ok(Self::Scheme),
            "vanguard" => Ok(Self::Vanguard),
            "token" => Ok(Self::Token),
            "double_faced_token" => Ok(Self::DoubleFacedToken),
            "emblem" => Ok(Self::Emblem),
            "augment" => Ok(Self::Augment),
            "host" => Ok(Self::Host),
            "art_series" => Ok(Self::ArtSeries),
            "reversible_card" => Ok(Self::ReversibleCard),
            other => Err(serde::de::Error::invalid_value(
                Unexpected::Str(other),
                &"Any valid card layout",
            )),
        }
    }
}
