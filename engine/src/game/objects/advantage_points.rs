// engine/src/game/objects/advantage_points.rs
//
// Advantage point calculator for race design validation.
//
// Direct port of:
//   RacePointsCalculator.java  — getAdvantagePoints(), getHabRangePoints()
//   Race.java                  — getPlanetHabitability()
// Source: stars-reborn-research/stars-4x/starsapi/.../craigstars/
//
// The raw point accumulator starts at STARTING_POINTS (1650) and is divided by
// 3 at the end.  A result ≥ 0 is a valid (balanced) race design; negative
// means the designer over-spent their budget.
//
// Hab values in our Race struct are in physical units (g, °C, mR/yr).  This
// module converts them to the 0-100 index space the original engine uses
// internally before running the sampling algorithm.

use super::race::{HabAxis, Lrt, Prt, Race, TechCost};

const STARTING_POINTS: i64 = 1650;

// ── Hab-index conversion ──────────────────────────────────────────────────────

/// Gravity_Map: index 0-100 → centi-g.  Mirrors race.rs decode table.
#[rustfmt::skip]
const GRAV_CENTI: [u16; 101] = [
     12,  12,  13,  13,  14,  14,  15,  15,  16,  17,  //   0-9
     17,  18,  19,  20,  21,  22,  24,  25,  27,  29,  //  10-19
     31,  33,  36,  40,  44,  50,  51,  52,  53,  54,  //  20-29
     55,  56,  58,  59,  60,  62,  64,  65,  67,  69,  //  30-39
     71,  73,  75,  78,  80,  83,  86,  89,  92,  96,  //  40-49
    100, 104, 108, 112, 116, 120, 124, 128, 132, 136,  //  50-59
    140, 144, 148, 152, 156, 160, 164, 168, 172, 176,  //  60-69
    180, 184, 188, 192, 196, 200, 224, 248, 272, 296,  //  70-79
    320, 344, 368, 392, 416, 440, 464, 488, 512, 536,  //  80-89
    560, 584, 608, 632, 656, 680, 704, 728, 752, 776,  //  90-99
    800,                                                 //    100
];

fn grav_to_idx(g: f64) -> i32 {
    let centi = (g * 100.0).round() as i32;
    GRAV_CENTI
        .iter()
        .enumerate()
        .min_by_key(|(_, &v)| (v as i32 - centi).unsigned_abs())
        .map(|(i, _)| i as i32)
        .unwrap_or(50)
}

fn temp_to_idx(temp: f64) -> i32 {
    (temp / 4.0 + 50.0).round().clamp(0.0, 100.0) as i32
}

fn rad_to_idx(rad: f64) -> i32 {
    rad.round().clamp(0.0, 100.0) as i32
}

/// Convert a HabAxis to (low_idx, high_idx) in 0-100 index space.
/// Returns (50, 50) for immune axes; the caller uses numIterations=1 for immune.
fn axis_to_idx(axis: &HabAxis, axis_type: usize) -> (i32, i32) {
    if axis.immune {
        return (50, 50);
    }
    let min = axis.min.unwrap_or(0.0);
    let max = axis.max.unwrap_or(100.0);
    match axis_type {
        0 => (grav_to_idx(min), grav_to_idx(max)),
        1 => (temp_to_idx(min), temp_to_idx(max)),
        _ => (rad_to_idx(min), rad_to_idx(max)),
    }
}

// ── Planet habitability ───────────────────────────────────────────────────────

/// Port of Race.getPlanetHabitability() from craigstars.
///
/// All values are in the 0-100 index space.
/// Returns 0-100 for habitable planets, negative (capped at -45) for red planets.
fn planet_habitability(
    is_immune: &[bool; 3],
    hab_low: &[i32; 3],
    hab_high: &[i32; 3],
    hab_center: &[i32; 3],
    planet_hab: &[i32; 3],
) -> i64 {
    let mut planet_value: i64 = 0;
    let mut red_value: i64 = 0;
    let mut ideality: i64 = 10000;

    for t in 0..3 {
        let pv = planet_hab[t];
        let lo = hab_low[t];
        let hi = hab_high[t];
        let ctr = hab_center[t];

        if is_immune[t] {
            planet_value += 10000;
        } else if lo <= pv && pv <= hi {
            // Green — planet is within habitable range.
            let (hab_radius, tmp) = if ctr > pv {
                (ctr - lo, ctr - pv)
            } else {
                (hi - ctr, pv - ctr)
            };
            let hab_radius = hab_radius.max(1); // guard against zero-width ranges
            let from_ideal = ((tmp * 100) / hab_radius).min(100);
            let poor_mod = tmp * 2 - hab_radius;
            let from_ideal = 100 - from_ideal;

            planet_value += (from_ideal * from_ideal) as i64;
            if poor_mod > 0 {
                ideality *= (hab_radius * 2 - poor_mod) as i64;
                ideality /= (hab_radius * 2) as i64;
            }
        } else {
            // Red — outside habitable range (capped at 15 per axis).
            let dist = if lo <= pv { pv - hi } else { lo - pv };
            red_value += dist.min(15) as i64;
        }
    }

    if red_value != 0 {
        return -red_value;
    }

    let pv = ((planet_value as f64 / 3.0).sqrt() + 0.9) as i64;
    pv * ideality / 10000
}

// ── Hab range sampling ────────────────────────────────────────────────────────

/// Port of RacePointsCalculator.getPlanetHabForHabIndex().
///
/// Returns the effective hab value for this sample point (post-terraform in loops
/// 1 and 2) and updates `tf_offset[hab_type]` with the residual terraform offset.
#[allow(clippy::too_many_arguments)]
fn planet_hab_for_index(
    iter_idx: i32,
    hab_type: usize,
    loop_idx: usize,
    num_iter: i32,
    hab_start: i32,
    hab_width: i32,
    hab_center: i32,
    is_immune: bool,
    tt_cf: i32,
    tf_offset: &mut [i32; 3],
) -> i32 {
    let tmp_hab = if iter_idx == 0 || num_iter <= 1 {
        hab_start
    } else {
        (hab_width * iter_idx) / (num_iter - 1) + hab_start
    };

    if loop_idx != 0 && !is_immune {
        let mut offset = hab_center - tmp_hab;
        if offset.abs() <= tt_cf {
            offset = 0;
        } else if offset < 0 {
            offset += tt_cf;
        } else {
            offset -= tt_cf;
        }
        tf_offset[hab_type] = offset;
        hab_center - offset
    } else {
        tmp_hab
    }
}

/// Port of RacePointsCalculator.getHabRangePoints().
///
/// Simulates planet habitability across the race's terraformable range in 3
/// outer loops (no-TF, low-TF, high-TF) and returns raw hab points.
fn get_hab_range_points(race: &Race) -> i64 {
    let is_immune = [
        race.hab.gravity.immune,
        race.hab.temperature.immune,
        race.hab.radiation.immune,
    ];
    let (glo, ghi) = axis_to_idx(&race.hab.gravity, 0);
    let (tlo, thi) = axis_to_idx(&race.hab.temperature, 1);
    let (rlo, rhi) = axis_to_idx(&race.hab.radiation, 2);

    let hab_low = [glo, tlo, rlo];
    let hab_high = [ghi, thi, rhi];
    let hab_center = [(glo + ghi) / 2, (tlo + thi) / 2, (rlo + rhi) / 2];
    let num_iter = [
        if is_immune[0] { 1 } else { 11 },
        if is_immune[1] { 1 } else { 11 },
        if is_immune[2] { 1 } else { 11 },
    ];

    let has_tt = race.lrts.contains(&Lrt::TT);
    let mut total: f64 = 0.0;

    for loop_idx in 0..3usize {
        let tt_cf: i32 = match loop_idx {
            0 => 0,
            1 => {
                if has_tt {
                    8
                } else {
                    5
                }
            }
            _ => {
                if has_tt {
                    17
                } else {
                    15
                }
            }
        };

        // Expand hab range by tt_cf on each side, clamped to [0, 100].
        let mut hab_start = [0i32; 3];
        let mut hab_width = [0i32; 3];
        for t in 0..3 {
            if is_immune[t] {
                hab_start[t] = 50;
                hab_width[t] = 11; // immune: constant width value used in Java
            } else {
                let lo = (hab_low[t] - tt_cf).max(0);
                let hi = (hab_high[t] + tt_cf).min(100);
                hab_start[t] = lo;
                hab_width[t] = hi - lo;
            }
        }

        let loop_weight: i64 = match loop_idx {
            0 => 7,
            1 => 5,
            _ => 6,
        };

        // tf_offset persists across all three axis loops within one outer loop.
        let mut tf_offset = [0i32; 3];

        let mut grav_sum: f64 = 0.0;
        for ig in 0..num_iter[0] {
            let gv = planet_hab_for_index(
                ig,
                0,
                loop_idx,
                num_iter[0],
                hab_start[0],
                hab_width[0],
                hab_center[0],
                is_immune[0],
                tt_cf,
                &mut tf_offset,
            );

            let mut temp_sum: f64 = 0.0;
            for it in 0..num_iter[1] {
                let tv = planet_hab_for_index(
                    it,
                    1,
                    loop_idx,
                    num_iter[1],
                    hab_start[1],
                    hab_width[1],
                    hab_center[1],
                    is_immune[1],
                    tt_cf,
                    &mut tf_offset,
                );

                let mut rad_sum: i64 = 0;
                for ir in 0..num_iter[2] {
                    let rv = planet_hab_for_index(
                        ir,
                        2,
                        loop_idx,
                        num_iter[2],
                        hab_start[2],
                        hab_width[2],
                        hab_center[2],
                        is_immune[2],
                        tt_cf,
                        &mut tf_offset,
                    );

                    let mut desirability = planet_habitability(
                        &is_immune,
                        &hab_low,
                        &hab_high,
                        &hab_center,
                        &[gv, tv, rv],
                    );

                    // Extra penalty when total terraform effort exceeds the loop budget.
                    let tf_sum: i32 = tf_offset.iter().sum();
                    if tf_sum > tt_cf {
                        desirability -= (tf_sum - tt_cf) as i64;
                        if desirability < 0 {
                            desirability = 0;
                        }
                    }

                    desirability = desirability * desirability * loop_weight;
                    rad_sum += desirability;
                }

                let rad_sum = if !is_immune[2] {
                    (rad_sum * hab_width[2] as i64) / 100
                } else {
                    rad_sum * 11
                };
                temp_sum += rad_sum as f64;
            }

            let temp_sum = if !is_immune[1] {
                temp_sum * hab_width[1] as f64 / 100.0
            } else {
                temp_sum * 11.0
            };
            grav_sum += temp_sum;
        }

        let grav_sum = if !is_immune[0] {
            grav_sum * hab_width[0] as f64 / 100.0
        } else {
            grav_sum * 11.0
        };
        total += grav_sum;
    }

    (total / 10.0 + 0.5) as i64
}

// ── PRT / LRT raw point costs ─────────────────────────────────────────────────

fn prt_raw_points(prt: &Prt) -> i64 {
    // From RacePointsCalculator.java prtPointCost static initializer.
    match prt {
        Prt::He => -40,
        Prt::Ss => -95,
        Prt::Wm => -45,
        Prt::Ca => -10,
        Prt::Is => 100,
        Prt::Sd => 150,
        Prt::Pp => -120,
        Prt::It => -180,
        Prt::Ar => -90,
        Prt::Joat => 66,
    }
}

fn lrt_raw_points(lrt: &Lrt) -> i64 {
    // From RacePointsCalculator.java lrtPointCost static initializer.
    // Negative = good for race (costs advantage points).
    // Positive = bad for race (earns advantage points).
    match lrt {
        Lrt::IFE => -235,
        Lrt::TT => -25,
        Lrt::ARM => -159,
        Lrt::ISB => -201,
        Lrt::GR => 40,
        Lrt::UR => -240,
        Lrt::MA => -155,
        Lrt::NRE => 160,
        Lrt::CE => 240,
        Lrt::OBRM => 255,
        Lrt::NAS => 325,
        Lrt::LSP => 180,
        Lrt::BET => 70,
        Lrt::RS => 30,
    }
}

// ── Main calculator ───────────────────────────────────────────────────────────

/// Compute the advantage point total for a race design.
///
/// Returns the number of remaining advantage points (raw accumulator ÷ 3).
/// A value ≥ 0 is a valid (balanced) race design that the original game accepts.
/// A negative value means the race design has over-spent its budget.
pub fn advantage_points(race: &Race) -> i32 {
    let mut points: i64 = STARTING_POINTS;

    // ── Habitat range ─────────────────────────────────────────────────────────
    let hab_points = get_hab_range_points(race) / 2000;

    // ── Growth rate ───────────────────────────────────────────────────────────
    // gr_raw: original GR% integer (1-20), saved for use in the factory penalty.
    // gr:     transformed value used in the hab×growth penalty below.
    let gr_raw = race.economy.growth_rate as i64;
    let mut gr = gr_raw;

    if gr <= 5 {
        points += (6 - gr) * 4200;
    } else if gr <= 13 {
        match gr {
            6 => points += 3600,
            7 => points += 2250,
            8 => points += 600,
            9 => points += 225,
            _ => {}
        }
        gr = gr * 2 - 5;
    } else if gr < 20 {
        gr = (gr - 6) * 3;
    } else {
        gr = 45;
    }

    // Habit range × growth penalty.
    points -= hab_points * gr / 24;

    // ── Hab off-center bonus & multi-immunity penalty ─────────────────────────
    let mut num_immunities: i64 = 0;
    let axes = [
        (&race.hab.gravity, axis_to_idx(&race.hab.gravity, 0)),
        (&race.hab.temperature, axis_to_idx(&race.hab.temperature, 1)),
        (&race.hab.radiation, axis_to_idx(&race.hab.radiation, 2)),
    ];
    for (axis, (lo, hi)) in &axes {
        if axis.immune {
            num_immunities += 1;
        } else {
            let center = (lo + hi) / 2;
            points += (center - 50).unsigned_abs() as i64 * 4;
        }
    }
    if num_immunities > 1 {
        points -= 150;
    }

    // ── Factory & population production penalty ───────────────────────────────
    // Penalty scales with GR when factories and population are both very high.
    {
        let mut op = race.economy.colonists_operate_factories as i64;
        let mut pp = race.economy.factory_production as i64;
        if op > 10 || pp > 10 {
            op = (op - 9).max(1);
            pp = (pp - 9).max(1);
            let fpc: i64 = if race.prt == Prt::He { 3 } else { 2 };
            pp *= fpc;
            // Use gr_raw (original GR, not the transformed value) as grRate.
            let penalty = (pp * op) as f64 * gr_raw as f64;
            if num_immunities >= 2 {
                points -= (penalty / 2.0) as i64;
            } else {
                points -= (penalty / 9.0) as i64;
            }
        }
    }

    // ── Population efficiency (colonists per resource) ────────────────────────
    {
        let pop_eff = (race.economy.resource_production / 100).min(25) as i64;
        if pop_eff <= 7 {
            points -= 2400;
        } else if pop_eff == 8 {
            points -= 1260;
        } else if pop_eff == 9 {
            points -= 600;
        } else if pop_eff > 10 {
            points += (pop_eff - 10) * 120;
        }
        // pop_eff == 10: baseline, no change.
    }

    // ── Factory design points ─────────────────────────────────────────────────
    if race.prt == Prt::Ar {
        // AR has no conventional factories/mines; fixed bonus.
        points += 210;
    } else {
        let factory_output = race.economy.factory_production as i64;
        let factory_cost = race.economy.factory_cost as i64;
        let num_factories = race.economy.colonists_operate_factories as i64;

        // prod_p > 0: output below standard (worse); cost_p > 0: cost below 10 (cheaper = better).
        let prod_p = 10 - factory_output;
        let cost_p = 10 - factory_cost;
        let oper_p = 10 - num_factories;

        let mut tmp: i64 = 0;
        tmp += if prod_p > 0 {
            prod_p * 100
        } else {
            prod_p * 121
        };
        tmp += if cost_p > 0 {
            cost_p * cost_p * -60
        } else {
            cost_p * -55
        };
        tmp += if oper_p > 0 { oper_p * 40 } else { oper_p * 35 };

        // Cap the total upward from very generous factory designs.
        const LLFP: i64 = 700;
        if tmp > LLFP {
            tmp = (tmp - LLFP) / 3 + LLFP;
        }

        // Extra penalties for extreme num_factories values.
        if oper_p <= -7 {
            if oper_p < -11 {
                if oper_p < -14 {
                    tmp -= 360;
                } else {
                    tmp += (oper_p + 7) * 45;
                }
            } else {
                tmp += (oper_p + 6) * 30;
            }
        }
        if prod_p <= -3 {
            tmp += (prod_p + 2) * 60;
        }

        points += tmp;

        if race.economy.factory_cheap_germanium {
            points -= 175;
        }

        // ── Mine design points ────────────────────────────────────────────────
        let mine_output = race.economy.mine_production as i64;
        let mine_cost = race.economy.mine_cost as i64;
        let num_mines = race.economy.colonists_operate_mines as i64;

        let prod_p = 10 - mine_output;
        let cost_p = 3 - mine_cost; // baseline for mine cost is 3, not 10
        let oper_p = 10 - num_mines;

        let mut tmp: i64 = 0;
        tmp += if prod_p > 0 {
            prod_p * 100
        } else {
            prod_p * 169
        };
        tmp += if cost_p > 0 {
            -360
        } else {
            cost_p * (-65) + 80
        };
        tmp += if oper_p > 0 { oper_p * 40 } else { oper_p * 35 };

        points += tmp;
    }

    // ── PRT raw points ────────────────────────────────────────────────────────
    points += prt_raw_points(&race.prt);

    // ── LRT raw points with count penalties ───────────────────────────────────
    // "good" LRTs have negative raw cost (beneficial for race, costs budget).
    // "bad"  LRTs have positive raw cost (penalty for race, earns budget).
    let mut bad_lrts: i64 = 0;
    let mut good_lrts: i64 = 0;
    for lrt in &race.lrts {
        let raw = lrt_raw_points(lrt);
        if raw >= 0 {
            bad_lrts += 1;
        } else {
            good_lrts += 1;
        }
        points += raw;
    }
    let total_lrts = good_lrts + bad_lrts;
    // Penalty for selecting more than 4 LRTs total.
    if total_lrts > 4 {
        points -= total_lrts * (total_lrts - 4) * 10;
    }
    // Penalty for imbalanced good/bad LRT ratios (>3 difference either way).
    if bad_lrts - good_lrts > 3 {
        points -= (bad_lrts - good_lrts - 3) * 60;
    }
    if good_lrts - bad_lrts > 3 {
        points -= (good_lrts - bad_lrts - 3) * 40;
    }

    // NAS has PRT-specific additional penalties.
    if race.lrts.contains(&Lrt::NAS) {
        points -= match race.prt {
            Prt::Pp => 280,
            Prt::Ss => 200,
            Prt::Joat => 40,
            _ => 0,
        };
    }

    // ── Research cost points ──────────────────────────────────────────────────
    // techcosts > 0: more Cheap fields than Expensive (penalized — too easy)
    // techcosts < 0: more Expensive fields than Cheap (rewarded — handicapped)
    let rc = &race.research_costs;
    let fields = [
        &rc.energy,
        &rc.weapons,
        &rc.propulsion,
        &rc.construction,
        &rc.electronics,
        &rc.biotechnology,
    ];
    let mut techcosts: i64 = 0;
    for f in &fields {
        match f {
            TechCost::Expensive => techcosts -= 1,
            TechCost::Cheap => techcosts += 1,
            TechCost::Normal => {}
        }
    }

    if techcosts > 0 {
        // More Cheap than Expensive: quadratic penalty.
        points -= techcosts * techcosts * 130;
        // Partial rebate for having all 5 or 6 fields Cheap (already heavily penalized).
        if techcosts >= 6 {
            points += 1430;
        } else if techcosts == 5 {
            points += 520;
        }
    } else if techcosts < 0 {
        // More Expensive than Cheap: stepped bonus (superlinear).
        const SCIENCE_COST: [i64; 6] = [150, 330, 540, 780, 1050, 1380];
        let idx = ((-techcosts - 1) as usize).min(5);
        points += SCIENCE_COST[idx];
        // Extra penalty for high-Expensive races with efficient population production.
        if techcosts < -4 && race.economy.resource_production < 1000 {
            points -= 190;
        }
    }

    // Expensive Tech Boost flag: starts Expensive fields at tech level 3.
    if race.research_costs.expensive_tech_start_at_3 {
        points -= 180;
    }

    // AR with Cheap energy: extra penalty (energy drives AR resource production strongly).
    if race.prt == Prt::Ar && race.research_costs.energy == TechCost::Cheap {
        points -= 100;
    }

    (points / 3) as i32
}
