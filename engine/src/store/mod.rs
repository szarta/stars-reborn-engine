// engine/src/store/mod.rs
//
// Game state persistence — one sub-directory per game under ENGINE_DATA_DIR.
//
// Layout:
//   {data_dir}/{game_uuid}/game.json   — full serialized GameState
//
// ENGINE_DATA_DIR env var overrides the default location.
// Default: $HOME/.local/share/stars-reborn/games  (Linux XDG convention)
//          ./engine_data                           (fallback if HOME unset)

use std::fs;
use std::path::{Path, PathBuf};

use log::{error, info, warn};

use crate::game::state::GameState;

// ---------------------------------------------------------------------------
// Data directory
// ---------------------------------------------------------------------------

pub fn data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("ENGINE_DATA_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("stars-reborn")
            .join("games");
    }
    PathBuf::from("./engine_data")
}

// ---------------------------------------------------------------------------
// Save
// ---------------------------------------------------------------------------

pub fn save_game(game: &GameState) -> Result<(), Box<dyn std::error::Error>> {
    let game_dir = data_dir().join(&game.id);
    fs::create_dir_all(&game_dir)?;
    let path = game_dir.join("game.json");
    let json = serde_json::to_string_pretty(game)?;
    fs::write(&path, json)?;
    info!("Saved game {} to {}", game.id, path.display());
    Ok(())
}

// ---------------------------------------------------------------------------
// Delete
// ---------------------------------------------------------------------------

/// Remove a game's on-disk directory.
///
/// Returns `Ok(true)` if the directory existed and was removed, `Ok(false)`
/// if no directory was present (idempotent — the caller may still treat the
/// request as successful if the in-memory copy was removed).
///
/// The caller is responsible for validating `game_id` to prevent path
/// traversal; this function joins it onto the data dir as-is.
pub fn delete_game(game_id: &str) -> std::io::Result<bool> {
    delete_game_in(&data_dir(), game_id)
}

fn delete_game_in(base: &Path, game_id: &str) -> std::io::Result<bool> {
    let game_dir = base.join(game_id);
    if !game_dir.exists() {
        return Ok(false);
    }
    fs::remove_dir_all(&game_dir)?;
    info!("Deleted game directory {}", game_dir.display());
    Ok(true)
}

// ---------------------------------------------------------------------------
// Load
// ---------------------------------------------------------------------------

pub fn load_all_games() -> Vec<GameState> {
    let dir = data_dir();
    if !dir.exists() {
        return vec![];
    }
    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(e) => {
            warn!("Could not read data dir {}: {}", dir.display(), e);
            return vec![];
        }
    };

    let mut games = Vec::new();
    for entry in entries.flatten() {
        let game_path = entry.path().join("game.json");
        if !game_path.exists() {
            continue;
        }
        match fs::read_to_string(&game_path) {
            Ok(content) => match serde_json::from_str::<GameState>(&content) {
                Ok(game) => {
                    info!("Loaded game {} ({})", game.id, game.name);
                    games.push(game);
                }
                Err(e) => error!("Failed to parse {}: {}", game_path.display(), e),
            },
            Err(e) => error!("Failed to read {}: {}", game_path.display(), e),
        }
    }
    games
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_test_base() -> PathBuf {
        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("stars-reborn-test-{pid}-{nanos}"))
    }

    #[test]
    fn delete_returns_false_when_directory_missing() {
        let base = unique_test_base();
        fs::create_dir_all(&base).unwrap();
        assert!(
            !delete_game_in(&base, "no-such-game").unwrap(),
            "missing game dir should return Ok(false), not error"
        );
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn delete_removes_existing_game_directory() {
        let base = unique_test_base();
        let game_id = "abcdef01-2345-6789-abcd-ef0123456789";
        let game_dir = base.join(game_id);
        fs::create_dir_all(game_dir.join("nested")).unwrap();
        fs::write(game_dir.join("game.json"), "{}").unwrap();
        fs::write(game_dir.join("nested/extra.txt"), "data").unwrap();

        assert!(game_dir.exists());
        assert!(delete_game_in(&base, game_id).unwrap());
        assert!(!game_dir.exists());

        // Idempotent: a second call returns Ok(false).
        assert!(!delete_game_in(&base, game_id).unwrap());

        let _ = fs::remove_dir_all(&base);
    }
}
