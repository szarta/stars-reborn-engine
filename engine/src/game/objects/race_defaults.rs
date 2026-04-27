// engine/src/game/objects/race_defaults.rs
//
// Runtime registry of the six predefined races shipped with the original
// Stars! game: Humanoid (JOAT), Insectoid (WM), Antetheral (SD),
// Rabbitoid (IT), Nucleotid (SS), Silicanoid (HE).
//
// All values are oracle-confirmed against the .r1 files in
// engine/tests/data/default_races/ and against the in-game race wizard
// (advantage_points totals match — see ../advantage_points.rs tests).

use super::race::{Economy, HabAxis, HabPreferences, Lrt, Prt, Race, ResearchCosts, TechCost};

// ── Public lookup ─────────────────────────────────────────────────────────────

/// Look up a predefined race by its singular name.
///
/// Match is case-insensitive. Returns `None` for any name not in the original
/// six. Custom (race-wizard) races are not predefined and must be supplied
/// inline by the caller.
pub fn default_race_by_name(name: &str) -> Option<Race> {
    match name.to_ascii_lowercase().as_str() {
        "humanoid" => Some(humanoid()),
        "insectoid" => Some(insectoid()),
        "antetheral" => Some(antetheral()),
        "rabbitoid" => Some(rabbitoid()),
        "nucleotid" => Some(nucleotid()),
        "silicanoid" => Some(silicanoid()),
        _ => None,
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn hab_ranged(min_i: u32, max_i: u32) -> HabAxis {
    HabAxis {
        immune: false,
        min: None,
        max: None,
        min_idx: Some(min_i),
        max_idx: Some(max_i),
    }
}

fn hab_immune() -> HabAxis {
    HabAxis {
        immune: true,
        min: None,
        max: None,
        min_idx: None,
        max_idx: None,
    }
}

// ── The six predefined races ──────────────────────────────────────────────────

/// Humanoid: the default JOAT race. Oracle: 25 leftover advantage points.
pub fn humanoid() -> Race {
    Race {
        format_version: 1,
        name: "Humanoid".into(),
        plural_name: "Humanoids".into(),
        prt: Prt::Joat,
        lrts: vec![],
        hab: HabPreferences {
            gravity: hab_ranged(15, 85),
            temperature: hab_ranged(15, 85),
            radiation: hab_ranged(15, 85),
        },
        economy: Economy {
            resource_production: 1000,
            factory_production: 10,
            factory_cost: 10,
            factory_cheap_germanium: false,
            colonists_operate_factories: 10,
            mine_production: 10,
            mine_cost: 5,
            colonists_operate_mines: 10,
            growth_rate: 15,
        },
        research_costs: ResearchCosts {
            energy: TechCost::Normal,
            weapons: TechCost::Normal,
            propulsion: TechCost::Normal,
            construction: TechCost::Normal,
            electronics: TechCost::Normal,
            biotechnology: TechCost::Normal,
            expensive_tech_boost: false,
        },
        leftover_spend: Default::default(),
        icon_index: 0,
    }
}

/// Insectoid: WM race. Oracle: 43 leftover advantage points.
pub fn insectoid() -> Race {
    Race {
        format_version: 1,
        name: "Insectoid".into(),
        plural_name: "Insectoids".into(),
        prt: Prt::Wm,
        lrts: vec![Lrt::ISB, Lrt::CE, Lrt::RS],
        hab: HabPreferences {
            gravity: HabAxis {
                immune: true,
                min: None,
                max: None,
                min_idx: None,
                max_idx: None,
            },
            temperature: HabAxis {
                immune: false,
                min: Some(-200.0),
                max: Some(200.0),
                min_idx: Some(0),
                max_idx: Some(100),
            },
            radiation: HabAxis {
                immune: false,
                min: Some(70.0),
                max: Some(100.0),
                min_idx: Some(70),
                max_idx: Some(100),
            },
        },
        economy: Economy {
            resource_production: 1000,
            factory_production: 10,
            factory_cost: 10,
            factory_cheap_germanium: false,
            colonists_operate_factories: 10,
            mine_production: 9,
            mine_cost: 10,
            colonists_operate_mines: 6,
            growth_rate: 10,
        },
        research_costs: ResearchCosts {
            energy: TechCost::Cheap,
            weapons: TechCost::Cheap,
            propulsion: TechCost::Cheap,
            construction: TechCost::Cheap,
            electronics: TechCost::Normal,
            biotechnology: TechCost::Expensive,
            expensive_tech_boost: false,
        },
        leftover_spend: Default::default(),
        icon_index: 3,
    }
}

/// Antetheral: SD race. Oracle: 7 leftover advantage points.
pub fn antetheral() -> Race {
    Race {
        format_version: 1,
        name: "Antetheral".into(),
        plural_name: "Antheherals".into(),
        prt: Prt::Sd,
        lrts: vec![Lrt::ARM, Lrt::MA, Lrt::NRE, Lrt::CE, Lrt::NAS],
        hab: HabPreferences {
            gravity: hab_ranged(0, 30),
            temperature: hab_ranged(0, 100),
            radiation: hab_ranged(70, 100),
        },
        economy: Economy {
            resource_production: 700,
            factory_production: 11,
            factory_cost: 10,
            factory_cheap_germanium: false,
            colonists_operate_factories: 18,
            mine_production: 10,
            mine_cost: 10,
            colonists_operate_mines: 10,
            growth_rate: 7,
        },
        research_costs: ResearchCosts {
            energy: TechCost::Cheap,
            weapons: TechCost::Expensive,
            propulsion: TechCost::Cheap,
            construction: TechCost::Cheap,
            electronics: TechCost::Cheap,
            biotechnology: TechCost::Cheap,
            expensive_tech_boost: false,
        },
        leftover_spend: Default::default(),
        icon_index: 17,
    }
}

/// Rabbitoid: IT race. Oracle: 32 leftover advantage points.
///
/// Note the explicit `min_idx`/`max_idx` on the gravity axis: the .r1 binary
/// stores raw index 10 (= 0.17 g), but `GRAV_CENTI[9] == GRAV_CENTI[10] == 17`,
/// so a naive physical→index round-trip would land on index 9 and shift the
/// hab center by one slot. See advantage_points::tests for the regression.
pub fn rabbitoid() -> Race {
    Race {
        format_version: 1,
        name: "Rabbitoid".into(),
        plural_name: "Rabbitoids".into(),
        prt: Prt::It,
        lrts: vec![Lrt::IFE, Lrt::TT, Lrt::CE, Lrt::NAS],
        hab: HabPreferences {
            gravity: HabAxis {
                immune: false,
                min: Some(0.17),
                max: Some(1.24),
                min_idx: Some(10),
                max_idx: Some(56),
            },
            temperature: HabAxis {
                immune: false,
                min: Some(-60.0),
                max: Some(124.0),
                min_idx: Some(35),
                max_idx: Some(81),
            },
            radiation: HabAxis {
                immune: false,
                min: Some(13.0),
                max: Some(53.0),
                min_idx: Some(13),
                max_idx: Some(53),
            },
        },
        economy: Economy {
            resource_production: 1000,
            factory_production: 10,
            factory_cost: 9,
            factory_cheap_germanium: true,
            colonists_operate_factories: 17,
            mine_production: 10,
            mine_cost: 9,
            colonists_operate_mines: 10,
            growth_rate: 20,
        },
        research_costs: ResearchCosts {
            energy: TechCost::Expensive,
            weapons: TechCost::Expensive,
            propulsion: TechCost::Cheap,
            construction: TechCost::Normal,
            electronics: TechCost::Normal,
            biotechnology: TechCost::Cheap,
            expensive_tech_boost: false,
        },
        leftover_spend: Default::default(),
        icon_index: 0,
    }
}

/// Nucleotid: SS race (gravity-immune). Oracle: 11 leftover advantage points.
pub fn nucleotid() -> Race {
    Race {
        format_version: 1,
        name: "Nucleotid".into(),
        plural_name: "Nucleotids".into(),
        prt: Prt::Ss,
        lrts: vec![Lrt::ARM, Lrt::ISB],
        hab: HabPreferences {
            gravity: hab_immune(),
            temperature: hab_ranged(12, 88),
            radiation: hab_ranged(0, 100),
        },
        economy: Economy {
            resource_production: 900,
            factory_production: 10,
            factory_cost: 10,
            factory_cheap_germanium: false,
            colonists_operate_factories: 10,
            mine_production: 10,
            mine_cost: 15,
            colonists_operate_mines: 5,
            growth_rate: 10,
        },
        research_costs: ResearchCosts {
            energy: TechCost::Expensive,
            weapons: TechCost::Expensive,
            propulsion: TechCost::Expensive,
            construction: TechCost::Expensive,
            electronics: TechCost::Expensive,
            biotechnology: TechCost::Expensive,
            expensive_tech_boost: true,
        },
        leftover_spend: Default::default(),
        icon_index: 24,
    }
}

/// Silicanoid: HE race (all-immune). Oracle: 9 leftover advantage points.
pub fn silicanoid() -> Race {
    Race {
        format_version: 1,
        name: "Silicanoid".into(),
        plural_name: "Silicanoids".into(),
        prt: Prt::He,
        lrts: vec![Lrt::IFE, Lrt::UR, Lrt::OBRM, Lrt::BET],
        hab: HabPreferences {
            gravity: hab_immune(),
            temperature: hab_immune(),
            radiation: hab_immune(),
        },
        economy: Economy {
            resource_production: 800,
            factory_production: 12,
            factory_cost: 12,
            factory_cheap_germanium: false,
            colonists_operate_factories: 15,
            mine_production: 10,
            mine_cost: 9,
            colonists_operate_mines: 10,
            growth_rate: 6,
        },
        research_costs: ResearchCosts {
            energy: TechCost::Normal,
            weapons: TechCost::Normal,
            propulsion: TechCost::Cheap,
            construction: TechCost::Cheap,
            electronics: TechCost::Normal,
            biotechnology: TechCost::Expensive,
            expensive_tech_boost: false,
        },
        leftover_spend: Default::default(),
        icon_index: 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_is_case_insensitive() {
        assert!(default_race_by_name("Humanoid").is_some());
        assert!(default_race_by_name("humanoid").is_some());
        assert!(default_race_by_name("HUMANOID").is_some());
        assert!(default_race_by_name("hUmAnOiD").is_some());
    }

    #[test]
    fn lookup_returns_none_for_unknown() {
        assert!(default_race_by_name("Vulcan").is_none());
        assert!(default_race_by_name("").is_none());
    }

    #[test]
    fn all_six_predefined_races_resolve() {
        for name in [
            "Humanoid",
            "Insectoid",
            "Antetheral",
            "Rabbitoid",
            "Nucleotid",
            "Silicanoid",
        ] {
            let r = default_race_by_name(name).expect(name);
            assert_eq!(r.name, name);
        }
    }
}
