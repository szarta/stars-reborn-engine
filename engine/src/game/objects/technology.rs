/// Technology data model
///
/// Mirrors the original game's technology class hierarchy.
/// Data values are sourced from the legacy Python engine (stars_reborn/data/factory.py);
/// all require oracle verification against stars.exe before the engine relies on them.
///
/// See stars-reborn-design/docs/mechanics/ship_design.rst
///     stars-reborn-design/docs/mechanics/starbase_design.rst
///     stars-reborn-design/docs/reference/engines.rst
///     stars-reborn-design/docs/reference/ship_hulls.rst
///     stars-reborn-design/docs/reference/starbase_hulls.rst

// ---------------------------------------------------------------------------
// Tech requirements and costs
// ---------------------------------------------------------------------------

/// Tech levels required to unlock a technology.
/// Order: [Energy, Weapons, Propulsion, Construction, Electronics, Bio]
#[derive(Debug, Clone, PartialEq)]
pub struct TechRequirements {
    pub energy: u8,
    pub weapons: u8,
    pub propulsion: u8,
    pub construction: u8,
    pub electronics: u8,
    pub bio: u8,
}

impl TechRequirements {
    pub const NONE: Self = Self {
        energy: 0,
        weapons: 0,
        propulsion: 0,
        construction: 0,
        electronics: 0,
        bio: 0,
    };
}

/// Mineral and resource cost to build one unit of a technology.
#[derive(Debug, Clone, PartialEq)]
pub struct TechCost {
    pub ironium: u32,
    pub boranium: u32,
    pub germanium: u32,
    pub resources: u32,
}

// ---------------------------------------------------------------------------
// Slot type bitmask
// ---------------------------------------------------------------------------

// Bitmask defining which component categories a hull slot accepts.
// A part may be placed in a slot if `slot_type & part_category != 0`.
bitflags::bitflags! {
    /// Bitmask defining which component categories a hull slot accepts.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SlotType: u16 {
        const BEAM_WEAPONS    = 0x0001;
        const TORPEDOES       = 0x0002;
        const ARMOR           = 0x0004;
        const SHIELDS         = 0x0008;
        const ELECTRICAL      = 0x0010;
        const MECHANICAL      = 0x0040;
        const BOMBS           = 0x0080;
        const MINE_LAYERS     = 0x0100;
        const MINING_ROBOTS   = 0x0200;
        const SCANNERS        = 0x0400;
        const ORBITAL         = 0x0800;
        const ENGINES         = 0x1000;
        const GENERAL_PURPOSE = 0x2000;

        // Composite slot types
        const WEAPONS              = Self::BEAM_WEAPONS.bits() | Self::TORPEDOES.bits();
        const PROTECTION           = Self::ARMOR.bits() | Self::SHIELDS.bits();
        const ORBITAL_ELECT        = Self::ORBITAL.bits() | Self::ELECTRICAL.bits();
        const SCANNER_ELECT_MECH   = Self::SCANNERS.bits() | Self::ELECTRICAL.bits() | Self::MECHANICAL.bits();
        const SHIELD_ELECT_MECH    = Self::SHIELDS.bits() | Self::ELECTRICAL.bits() | Self::MECHANICAL.bits();
        const MINE_ELECT_MECH      = Self::MINE_LAYERS.bits() | Self::ELECTRICAL.bits() | Self::MECHANICAL.bits();
        const WEAPON_SHIELD        = Self::SHIELDS.bits() | Self::WEAPONS.bits();
        const ELECT_MECH           = Self::ELECTRICAL.bits() | Self::MECHANICAL.bits();
        const ARMOR_SCANNER_ELECT_MECH = Self::ARMOR.bits() | Self::SCANNERS.bits()
                                       | Self::ELECTRICAL.bits() | Self::MECHANICAL.bits();
    }
}

/// One slot on a hull: what it accepts and how many parts it holds.
#[derive(Debug, Clone)]
pub struct TechSlot {
    pub slot_type: SlotType,
    pub max_count: u8,
}

// ---------------------------------------------------------------------------
// Base types
// ---------------------------------------------------------------------------

/// Properties common to all installable parts.
#[derive(Debug, Clone)]
pub struct PartBase {
    pub requirements: TechRequirements,
    pub cost: TechCost,
    pub mass: u32,
}

/// Properties common to non-installable planetary/orbital technologies.
#[derive(Debug, Clone)]
pub struct TechBase {
    pub requirements: TechRequirements,
    pub cost: TechCost,
}

// ---------------------------------------------------------------------------
// Ship and starbase hulls
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ShipHull {
    pub requirements: TechRequirements,
    pub cost: TechCost,
    pub slots: Vec<TechSlot>,
    pub armor: u32,
    pub initiative: u8,
    pub mass: u32,
    pub fuel_capacity: u32,
    pub cargo_capacity: u32,
    /// Hull-intrinsic fuel regeneration per year (fuel transport hulls only).
    pub fuel_regen_per_year: u32,
    /// Additional fuel regeneration as % of current fuel (fuel transport hulls only).
    pub fuel_regen_percent: u8,
}

#[derive(Debug, Clone)]
pub struct StarbaseHull {
    pub requirements: TechRequirements,
    pub cost: TechCost,
    pub slots: Vec<TechSlot>,
    pub armor: u32,
    pub initiative: u8,
    /// Max ship mass (kT) that can be built or repaired; None = unlimited.
    pub dock_capacity: Option<u32>,
}

// ---------------------------------------------------------------------------
// Engines
// ---------------------------------------------------------------------------

/// Fuel consumption (mg/ly) at each warp speed 0–10.
pub type FuelTable = [u32; 11];

#[derive(Debug, Clone)]
pub struct Engine {
    pub base: PartBase,
    /// Fuel consumed per light-year at warp speed index (0–10).
    pub fuel_table: FuelTable,
    /// Battle speed rating (fractional halves; e.g., 10 = speed 5).
    pub battle_speed: u8,
    /// Whether this engine can safely travel at warp 10.
    pub warp10_travel: bool,
    /// Highest warp with zero fuel cost (ram scoop characteristic).
    pub last_free_warp: u8,
    /// Cloaking bonus percentage (Enigma Pulsar only).
    pub cloaking: u8,
    /// Additive battle speed modifier (Enigma Pulsar: +0.25).
    pub battle_speed_modifier: f32,
}

// ---------------------------------------------------------------------------
// Weapons
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct BeamWeapon {
    pub base: PartBase,
    pub power: u32,
    pub range: u8,
    pub initiative: u8,
    /// Hits all targets in range simultaneously.
    pub spread: bool,
    /// Damages shields only; cannot damage armor.
    pub shields_only: bool,
}

#[derive(Debug, Clone)]
pub struct Torpedo {
    pub base: PartBase,
    pub power: u32,
    pub range: u8,
    pub initiative: u8,
    /// Base hit probability (0.0–1.0) before ECM modifiers.
    pub accuracy: f32,
}

// ---------------------------------------------------------------------------
// Armor and shields
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Armor {
    pub base: PartBase,
    pub armor_value: u32,
    /// Some armor pieces also provide shield value (e.g., Fielded Kelarium).
    pub shield_value: u32,
    /// Cloaking bonus % (e.g., Depleted Neutronium: 25%).
    pub cloaking: u8,
}

#[derive(Debug, Clone)]
pub struct Shield {
    pub base: PartBase,
    pub shield_value: u32,
    /// Some shields also provide armor (e.g., Croby Sharmor).
    pub armor_value: u32,
    /// Cloaking bonus % (e.g., Shadow Shield: 35%).
    pub cloaking: u8,
    /// Jamming bonus % (e.g., Langston Shell: 5%).
    pub jamming: u8,
    /// Ship scanner basic range (e.g., Langston Shell: 50 ly).
    pub scanner_basic_range: u32,
    /// Ship scanner penetrating range (e.g., Langston Shell: 25 ly).
    pub scanner_penetrating_range: u32,
}

// ---------------------------------------------------------------------------
// Bombs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Bomb {
    pub base: PartBase,
    /// Fraction of colonists killed per bomb (e.g., 0.012 = 1.2%).
    pub colonist_kill_percent: f32,
    /// Minimum colonists killed even if percent would give fewer.
    pub minimum_colonists_killed: u32,
    pub buildings_destroyed: u32,
    /// Smart bombs skip defenses more effectively.
    pub smart: bool,
}

// ---------------------------------------------------------------------------
// Mine layers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum MineType {
    Normal,
    Heavy,
    Speed,
}

#[derive(Debug, Clone)]
pub struct MineLayer {
    pub base: PartBase,
    pub mines_per_year: u32,
    pub mine_type: MineType,
    /// Minimum warp speed that is safe through this mine field.
    pub min_safe_warp: u8,
    /// Probability of hitting a mine per light-year of travel.
    pub hit_chance_per_ly: f32,
    pub damage_ship_no_ram_scoop: u32,
    pub damage_ship_ram_scoop: u32,
    pub min_damage_fleet_no_ram_scoop: u32,
    pub min_damage_fleet_ram_scoop: u32,
}

// ---------------------------------------------------------------------------
// Scanners
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Scanner {
    pub base: PartBase,
    pub basic_range: u32,
    pub penetrating_range: u32,
    /// Cloaking bonus % (e.g., Chameleon Scanner: 20%).
    pub cloaking: u8,
}

// ---------------------------------------------------------------------------
// Mechanical parts
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Mechanical {
    pub base: PartBase,
    /// Extra cargo capacity added to the ship (cargo pods).
    pub cargo: u32,
    /// Extra fuel capacity added to the ship (fuel tanks).
    pub fuel: u32,
    /// Battle speed modifier (maneuvering jets: +0.25, overthruster: +0.5).
    pub battle_speed_modifier: f32,
    /// Beam damage reduction % (beam deflector: 10%).
    pub beam_reduction: u8,
    /// Cloaking bonus % (Multi-Cargo Pod: 10%).
    pub cloaking: u8,
    /// Armor value (Multi-Cargo Pod: 50).
    pub armor_value: u32,
    /// For JumpGate: enables stargate-like one-way jump for IT races.
    pub is_jump_gate: bool,
}

// ---------------------------------------------------------------------------
// Electrical parts
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Electrical {
    pub base: PartBase,
    /// Cloaking bonus %.
    pub cloaking: u8,
    /// Torpedo accuracy bonus % (battle computers).
    pub torpedo_accuracy: u8,
    /// Combat initiative bonus (battle computers).
    pub initiative: u8,
    /// Torpedo miss probability reduction % (jammers).
    pub jamming: u8,
    /// Beam weapon power bonus % (capacitors).
    pub beam_damage: u8,
    /// Hull-level fuel tank (Anti-Matter Generator).
    pub fuel: u32,
    /// Anti-Matter Generator annual fuel production.
    pub fuel_per_year: u32,
    /// Battle speed modifier (Multi-Function Pod: +0.25).
    pub battle_speed_modifier: f32,
    /// Whether this item dampens enemy energy weapons.
    pub is_energy_dampener: bool,
    /// Whether this item detects cloaked ships.
    pub is_tachyon_detector: bool,
}

// ---------------------------------------------------------------------------
// Mining robots
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct MiningRobot {
    pub base: PartBase,
    /// kT of each mineral extracted per year per robot.
    pub mining_value: u32,
    /// Cloaking bonus % (Alien Miner: 30%).
    pub cloaking: u8,
    /// Jamming bonus % (Alien Miner: 30%).
    pub jamming: u8,
    /// Battle speed modifier (Alien Miner: +0.125).
    pub battle_speed_modifier: f32,
}

// ---------------------------------------------------------------------------
// Planetary technologies (not installable on ships)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PlanetaryScanner {
    pub base: TechBase,
    pub basic_range: u32,
    pub penetrating_range: u32,
}

#[derive(Debug, Clone)]
pub struct PlanetaryDefense {
    pub base: TechBase,
    /// Per-defense kill probability against a single bomb (0.0–1.0).
    pub base_coverage: f32,
}

#[derive(Debug, Clone)]
pub struct Terraforming {
    pub base: TechBase,
    /// Max hab units shiftable per year in gravity axis.
    pub gravity: i8,
    /// Max hab units shiftable per year in temperature axis.
    pub temperature: i8,
    /// Max hab units shiftable per year in radiation axis.
    pub radiation: i8,
}

// ---------------------------------------------------------------------------
// Orbital structures (starbase-installable only)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Stargate {
    pub base: TechBase,
    /// Max ship mass (kT) transferable safely; None = unlimited.
    pub safe_mass: Option<u32>,
    /// Max transfer distance (ly) safely; None = unlimited.
    pub safe_range: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct MassDriver {
    pub base: TechBase,
    /// Packet launch velocity (warp).
    pub warp: u8,
}

// ---------------------------------------------------------------------------
// Special / Mystery Trader items
// ---------------------------------------------------------------------------

/// Genesis Device — one-time use; exact effect TBD from oracle research.
#[derive(Debug, Clone)]
pub struct GenesisDevice {
    pub base: TechBase,
}

// ---------------------------------------------------------------------------
// TODO: Data loading
// ---------------------------------------------------------------------------
//
// The full technology catalog (all ~300 items with exact numeric values) must
// be loaded from a data file at engine startup. Candidate formats:
//   - JSON (consistent with turn file format)
//   - Hard-coded Rust constants (simpler, avoids file I/O)
//
// Source values are in the legacy Python engine (stars_reborn/data/factory.py,
// now removed). All values need oracle verification before use. See:
//   stars-reborn-research/PLAN.md   — research tasks
//   stars-reborn-design/docs/reference/engines.rst
//   stars-reborn-design/docs/reference/ship_hulls.rst
//   stars-reborn-design/docs/reference/starbase_hulls.rst
