// engine/src/game/score.rs
//
// F10 score computation — pure functions, no game-state dependency.
//
// See stars-reborn-design/docs/mechanics/victory.rst for the full formula
// and empirical validation (22 player-snapshots, zero residual).

/// Per-planet score contribution. 1 pt floor for any colonized planet,
/// +1 pt per started 100k colonists, capped at 6.
pub fn planet_points(population: u32) -> u32 {
    if population == 0 {
        return 1;
    }
    let bonus = population.div_ceil(100_000);
    bonus.clamp(1, 6)
}

/// Starbase points: 3 pts per starbase (Orbital Forts excluded by caller).
pub fn starbase_points(n_starbases: u32) -> u32 {
    3 * n_starbases
}

/// Unarmed ships (power rating 0): 0.5 pt each, count capped at planets owned.
pub fn unarmed_points(n_unarmed: u32, n_planets: u32) -> u32 {
    n_unarmed.min(n_planets) / 2
}

/// Escort ships (power 1..=1999): 2 pts each, count capped at planets owned.
pub fn escort_points(n_escort: u32, n_planets: u32) -> u32 {
    2 * n_escort.min(n_planets)
}

/// Capital ships (power >= 2000): harmonic-mean total, count capped at planets.
///
/// `floor(8 × N × P / (N + P))` where N is the capped capital-ship count and
/// P is the planet count. This is a single pool-wide total, not per-ship.
pub fn capital_points(n_capital: u32, n_planets: u32) -> u32 {
    let n = n_capital.min(n_planets);
    if n == 0 || n_planets == 0 {
        return 0;
    }
    let num = 8u64 * n as u64 * n_planets as u64;
    let denom = n as u64 + n_planets as u64;
    (num / denom) as u32
}

/// Score from a single tech-level value: 1/2/3/4 pts per level at the
/// 1–3 / 4–6 / 7–9 / 10+ breakpoints, summed over all levels gained.
pub fn tech_field_points(level: u32) -> u32 {
    let mut pts = 0;
    for l in 1..=level {
        pts += match l {
            1..=3 => 1,
            4..=6 => 2,
            7..=9 => 3,
            _ => 4,
        };
    }
    pts
}

/// Resource points: `floor(annual_resources / 30)`.
pub fn resource_points(annual_resources: u32) -> u32 {
    annual_resources / 30
}

/// Compute the total F10 score from pre-classified inputs.
///
/// Ship counts must already be classified by power rating
/// (0 → unarmed, 1..=1999 → escort, >=2000 → capital).
pub fn total_score(
    planet_populations: &[u32],
    starbase_count: u32,
    unarmed_count: u32,
    escort_count: u32,
    capital_count: u32,
    tech: [u32; 6],
    annual_resources: u32,
) -> u32 {
    let n_planets = planet_populations.len() as u32;
    let plt: u32 = planet_populations.iter().copied().map(planet_points).sum();
    let tech_pts: u32 = tech.iter().copied().map(tech_field_points).sum();
    plt + starbase_points(starbase_count)
        + unarmed_points(unarmed_count, n_planets)
        + escort_points(escort_count, n_planets)
        + capital_points(capital_count, n_planets)
        + tech_pts
        + resource_points(annual_resources)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planet_points_breakpoints() {
        assert_eq!(planet_points(0), 1);
        assert_eq!(planet_points(1), 1);
        assert_eq!(planet_points(99_999), 1);
        assert_eq!(planet_points(100_000), 1);
        assert_eq!(planet_points(100_001), 2);
        assert_eq!(planet_points(108_000), 2);
        assert_eq!(planet_points(393_000), 4);
        assert_eq!(planet_points(500_001), 6);
        assert_eq!(planet_points(10_000_000), 6);
    }

    #[test]
    fn tech_field_breakpoints() {
        assert_eq!(tech_field_points(0), 0);
        assert_eq!(tech_field_points(3), 3);
        assert_eq!(tech_field_points(6), 9);
        assert_eq!(tech_field_points(9), 18);
        assert_eq!(tech_field_points(10), 22);
        assert_eq!(tech_field_points(8), 1 + 1 + 1 + 2 + 2 + 2 + 3 + 3);
    }

    #[test]
    fn capital_harmonic_mean_floor() {
        assert_eq!(capital_points(0, 10), 0);
        assert_eq!(capital_points(5, 0), 0);
        assert_eq!(capital_points(1, 5), 6);
        assert_eq!(capital_points(26, 212), 185);
        assert_eq!(capital_points(20, 30), 96);
    }

    #[test]
    fn capital_count_capped_at_planets() {
        assert_eq!(capital_points(100, 10), capital_points(10, 10));
    }

    /// try2 year 2504 P1 Timmune (AR): 5 planets, 5 SB, 71 unarmed, 8 escort,
    /// 1 capital, tech 12/12/10/13/8/5, 3807 res → F10 score 318 (pop bonus 16).
    #[test]
    fn try2_year_2504_p1_timmune_ar() {
        let n_planets = 5;
        let tech_sum: u32 = [12u32, 12, 10, 13, 8, 5]
            .iter()
            .copied()
            .map(tech_field_points)
            .sum();
        let non_planet = starbase_points(5)
            + unarmed_points(71, n_planets)
            + escort_points(8, n_planets)
            + capital_points(1, n_planets)
            + tech_sum
            + resource_points(3807);
        assert_eq!(capital_points(1, n_planets), 6);
        assert_eq!(non_planet + n_planets + 16, 318);
    }

    /// try2 year 2504 P4 Hicardi (HE): 212 planets, 69 SB, 54 unarmed, 1209 escort,
    /// 26 capital, tech 15/24/20/16/19/12, 99 474 res → F10 score 5116 (pop bonus 430).
    #[test]
    fn try2_year_2504_p4_hicardi_he() {
        let n_planets = 212;
        let tech_sum: u32 = [15u32, 24, 20, 16, 19, 12]
            .iter()
            .copied()
            .map(tech_field_points)
            .sum();
        let non_planet = starbase_points(69)
            + unarmed_points(54, n_planets)
            + escort_points(1209, n_planets)
            + capital_points(26, n_planets)
            + tech_sum
            + resource_points(99_474);
        assert_eq!(capital_points(26, n_planets), 185);
        assert_eq!(non_planet + n_planets + 430, 5116);
    }
}
