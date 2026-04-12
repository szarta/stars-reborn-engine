// engine/src/game/objects/race.rs
//
// Race model: Primary Racial Trait, Lesser Racial Traits, habitat preferences,
// economy parameters, and research costs.
//
// Field offsets in the Stars! .r1 binary are confirmed via differential analysis
// of the six default race files (decrypt_stars.py, analyze_r1.py in
// stars-reborn-research).  See stars-reborn-design docs/new_game/race_file_format.rst.

use serde::{Deserialize, Serialize};

// ── Primary Racial Trait ──────────────────────────────────────────────────────

/// Primary Racial Trait.  Byte value in .r1: HE=0, SS=1, WM=2, CA=3, IS=4,
/// SD=5, PP=6, IT=7, AR=8, JOAT=9.  CA=3 and AR=8 are inferred; see R1.1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Prt {
    #[serde(rename = "HE")]
    He,
    #[serde(rename = "SS")]
    Ss,
    #[serde(rename = "WM")]
    Wm,
    #[serde(rename = "CA")]
    Ca,
    #[serde(rename = "IS")]
    Is,
    #[serde(rename = "SD")]
    Sd,
    #[serde(rename = "PP")]
    Pp,
    #[serde(rename = "IT")]
    It,
    #[serde(rename = "AR")]
    Ar,
    #[serde(rename = "JOAT")]
    Joat,
}

impl Prt {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::He),
            1 => Some(Self::Ss),
            2 => Some(Self::Wm),
            3 => Some(Self::Ca),
            4 => Some(Self::Is),
            5 => Some(Self::Sd),
            6 => Some(Self::Pp),
            7 => Some(Self::It),
            8 => Some(Self::Ar),
            9 => Some(Self::Joat),
            _ => None,
        }
    }
}

// ── Lesser Racial Traits ──────────────────────────────────────────────────────

/// Lesser Racial Trait.  Bit positions in the .r1 LRT bitmask (bytes 78-79)
/// have been identified but the exact byte encoding is not yet confirmed.
/// See research task R1.2.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lrt {
    NRE,  // No Ramscoop Engines     – bit 0
    IFE,  // Improved Fuel Efficiency – bit 1
    CE,   // Cheap Engines            – bit 2
    TT,   // Total Terraforming       – bit 3
    OBRM, // Only Basic Remote Mining – bit 4
    ARM,  // Advanced Remote Mining   – bit 5
    NAS,  // No Advanced Scanners     – bit 6
    ISB,  // Improved Starbases       – bit 7
    LSP,  // Low Starting Population  – bit 8
    GR,   // Generalized Research     – bit 9
    BET,  // Bleeding Edge Technology – bit 10
    UR,   // Ultimate Recycling       – bit 11
    RS,   // Regenerating Shields     – bit 12
    MA,   // Mineral Alchemy          – bit 13
}

// ── Research cost multiplier ──────────────────────────────────────────────────

/// Per-field research cost multiplier.  Stored as a byte in the .r1 struct:
/// 0 = Expensive, 1 = Normal, 2 = Cheap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TechCost {
    Expensive,
    Normal,
    Cheap,
}

impl TechCost {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::Expensive),
            1 => Some(Self::Normal),
            2 => Some(Self::Cheap),
            _ => None,
        }
    }
}

// ── Habitat preferences ───────────────────────────────────────────────────────

/// One habitat axis (gravity, temperature, or radiation).
/// If `immune` is true, `min` and `max` are absent.
/// Units: gravity in g, temperature in °C, radiation in mR/yr (0–100).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabAxis {
    pub immune: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

impl HabAxis {
    pub fn immune() -> Self {
        Self {
            immune: true,
            min: None,
            max: None,
        }
    }
    pub fn range(min: f64, max: f64) -> Self {
        Self {
            immune: false,
            min: Some(min),
            max: Some(max),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabPreferences {
    pub gravity: HabAxis,
    pub temperature: HabAxis,
    pub radiation: HabAxis,
}

// ── Economy ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Economy {
    pub resource_production: u32,
    pub factory_production: u32,
    pub factory_cost: u32,
    pub factory_cheap_germanium: bool,
    pub colonists_operate_factories: u32,
    pub mine_production: u32,
    pub mine_cost: u32,
    pub colonists_operate_mines: u32,
    pub growth_rate: u32,
}

// ── Research costs ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchCosts {
    pub energy: TechCost,
    pub weapons: TechCost,
    pub propulsion: TechCost,
    pub construction: TechCost,
    pub electronics: TechCost,
    pub biotechnology: TechCost,
}

// ── Race ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Race {
    pub format_version: u32,
    pub name: String,
    pub plural_name: String,
    pub prt: Prt,
    pub lrts: Vec<Lrt>,
    pub hab: HabPreferences,
    pub economy: Economy,
    pub research_costs: ResearchCosts,
    pub icon_index: u32,
}
