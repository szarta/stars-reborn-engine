// engine/tests/player_view_leak.rs
//
// Integration tests for the GameState → PlayerView choke point.
//
// These tests are the safety net for fog-of-war leaks. Any host-only field
// (gravity, concentrations, population, owner of an unobserved planet, etc.)
// that escapes through `player_view()` is a leak; serialising raw
// `GameState` to a non-host client is a bug.
//
// The tests exercise the public shape of `PlayerView` both as a Rust value
// and after JSON serialisation — the JSON form is what actually reaches a
// client, so we check unobserved planets contain only identity fields in
// the wire payload.

use stars_engine::game::objects::player::{Player, TechLevels};
use stars_engine::game::objects::race_defaults::humanoid;
use stars_engine::game::state::GameState;
use stars_engine::game::universe::generate_universe;
use stars_engine::game::view::{player_view, PlanetIntel, PlayerView};

const PLAYER_0: u32 = 0;
const PLAYER_1: u32 = 1;

/// Build a deterministic two-player GameState whose homeworlds are the two
/// lowest-id planets. Mirrors the homeworld assignment in `create_game`.
fn two_player_game() -> GameState {
    let generated = generate_universe(0, 1, 2, Some(42));
    let universe = generated.universe;
    let mut planet_states = generated.planet_states;

    let mut planet_ids: Vec<u32> = universe.planets.keys().copied().collect();
    planet_ids.sort();

    let hw0 = planet_ids[0];
    let hw1 = planet_ids[1];

    let race = humanoid();
    for (pid, hw_id) in [(PLAYER_0, hw0), (PLAYER_1, hw1)] {
        let s = planet_states.get_mut(&hw_id).unwrap();
        s.homeworld = true;
        s.owner = Some(pid);
        s.population = 25_000;
        s.factories = 10;
        s.mines = 10;
        s.set_homeworld_hab(&race);
    }

    GameState {
        id: "test-game".to_string(),
        name: "Leak Test".to_string(),
        universe,
        planet_states,
        players: vec![
            Player {
                id: PLAYER_0,
                race_name: "Humanoid".to_string(),
                homeworld_id: Some(hw0),
                tech: TechLevels::default(),
                race: Some(humanoid()),
            },
            Player {
                id: PLAYER_1,
                race_name: "Humanoid".to_string(),
                homeworld_id: Some(hw1),
                tech: TechLevels::default(),
                race: Some(humanoid()),
            },
        ],
        year: 2400,
    }
}

fn find_planet(view: &PlayerView, id: u32) -> &PlanetIntel {
    view.planets
        .iter()
        .find(|p| match p {
            PlanetIntel::Observed(o) => o.id == id,
            PlanetIntel::Unobserved(u) => u.id == id,
        })
        .expect("planet present in view")
}

#[test]
fn own_homeworld_is_observed_with_full_state() {
    let game = two_player_game();
    let hw0 = game.players[0].homeworld_id.unwrap();
    let view = player_view(&game, PLAYER_0);

    match find_planet(&view, hw0) {
        PlanetIntel::Observed(o) => {
            assert_eq!(o.owner, Some(PLAYER_0));
            assert_eq!(o.population, 25_000);
            assert_eq!(o.factories, 10);
            assert_eq!(o.mines, 10);
            assert!(o.homeworld);
            assert_eq!(o.years_since_last_scan, 0);
            // Homeworld hab is overridden to race centre, so the value of
            // the homeworld for its owning race must be the maximum 100%.
            assert_eq!(
                o.value, 100,
                "homeworld hab is centred on race tolerance — value should be 100%"
            );
        }
        PlanetIntel::Unobserved(_) => panic!("own homeworld must be Observed"),
    }
}

#[test]
fn opponent_homeworld_is_unobserved() {
    let game = two_player_game();
    let hw1 = game.players[1].homeworld_id.unwrap();
    let view = player_view(&game, PLAYER_0);

    match find_planet(&view, hw1) {
        PlanetIntel::Unobserved(u) => {
            assert_eq!(u.id, hw1);
            // identity is OK to expose — comes from .xy
        }
        PlanetIntel::Observed(_) => {
            panic!("opponent homeworld must NOT be observable to player 0")
        }
    }
}

#[test]
fn every_unowned_planet_is_unobserved_for_player_zero() {
    let game = two_player_game();
    let view = player_view(&game, PLAYER_0);
    let hw0 = game.players[0].homeworld_id.unwrap();

    for intel in &view.planets {
        match intel {
            PlanetIntel::Observed(o) => {
                assert_eq!(
                    o.id, hw0,
                    "only player 0's homeworld should be Observed; got id {}",
                    o.id
                );
            }
            PlanetIntel::Unobserved(_) => {}
        }
    }
}

#[test]
fn unobserved_planets_serialize_without_host_fields() {
    let game = two_player_game();
    let view = player_view(&game, PLAYER_0);

    let json = serde_json::to_value(&view).expect("serialise");
    let planets = json["planets"].as_array().expect("planets is array");

    let host_only_keys = [
        "gravity",
        "temperature",
        "radiation",
        "ironium_concentration",
        "boranium_concentration",
        "germanium_concentration",
        "surface_ironium",
        "surface_boranium",
        "surface_germanium",
        "population",
        "factories",
        "mines",
        "homeworld",
        "owner",
        "years_since_last_scan",
        "value",
    ];

    let hw0 = game.players[0].homeworld_id.unwrap();

    for p in planets {
        let id = p["id"].as_u64().expect("id present") as u32;
        let obj = p.as_object().expect("planet is object");

        if id == hw0 {
            // Own homeworld must carry full state on the wire.
            for key in host_only_keys {
                assert!(
                    obj.contains_key(key),
                    "own homeworld should serialise field `{key}`"
                );
            }
        } else {
            // Every other planet is Unobserved → only identity may appear.
            assert_eq!(
                obj.len(),
                4,
                "Unobserved planet {id} should have exactly 4 keys, got {:?}",
                obj.keys().collect::<Vec<_>>()
            );
            for key in host_only_keys {
                assert!(
                    !obj.contains_key(key),
                    "Unobserved planet {id} leaked host-only field `{key}` in JSON"
                );
            }
            for key in ["id", "name", "x", "y"] {
                assert!(
                    obj.contains_key(key),
                    "Unobserved planet {id} missing identity field `{key}`"
                );
            }
        }
    }
}

#[test]
fn view_planet_count_matches_universe() {
    let game = two_player_game();
    let view = player_view(&game, PLAYER_0);
    assert_eq!(view.planets.len(), game.universe.planets.len());
}

#[test]
fn view_planets_are_sorted_by_id() {
    let game = two_player_game();
    let view = player_view(&game, PLAYER_0);
    let ids: Vec<u32> = view
        .planets
        .iter()
        .map(|p| match p {
            PlanetIntel::Observed(o) => o.id,
            PlanetIntel::Unobserved(u) => u.id,
        })
        .collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted, "view.planets must be sorted by id");
}

#[test]
fn unknown_player_id_yields_empty_homeworld() {
    let game = two_player_game();
    // Defensive: a player_id with no matching Player should not panic and
    // should produce a view with homeworld_id None and every planet
    // Unobserved (no leakage to a phantom caller).
    let view = player_view(&game, 99);
    assert!(view.homeworld_id.is_none());
    for intel in &view.planets {
        assert!(
            matches!(intel, PlanetIntel::Unobserved(_)),
            "phantom caller must see only Unobserved planets"
        );
    }
}
