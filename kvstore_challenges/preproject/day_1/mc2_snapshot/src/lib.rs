// ### Mini-Challenge 2
//
// * **Name:** JSON snapshot round-trip (map ↔ disk) with error propagation
// * **Goal:** Build the muscle memory for persistence (save/load) without touching the real repo.
// * **Setup (scratch folder)**
//
//   * `cargo new mc2_snapshot && cd mc2_snapshot`
//   * Add deps: `serde`, `serde_json` (derive enabled)
// * **Requirements**
//
//   * Use `type Map = std::collections::HashMap<String,String>;`
//   * Implement in `src/lib.rs`:
//
//     * `fn save(path: &std::path::Path, map: &Map) -> Result<(), MyError>`
//     * `fn load(path: &std::path::Path) -> Result<Map, MyError>`
//   * Behavior rules:
//
//     * If file does not exist on load: return `Ok(empty_map)` (first-run behavior)
//     * If JSON is invalid: return a typed error (no panics)
//   * Add tests using a temp directory (or write to `target/` with a unique filename):
//
//     * round-trip: save then load equals original map
//     * corrupt file: write invalid JSON then `load` returns `Err(...)`
// * **Proof**
//
//   * `cargo test` passes and shows both persistence tests
// * **Guardrails**
//
//   * No `unwrap/expect` in `src/lib.rs`
//   * Log “save/load attempted” to stderr from `main.rs` only (keep lib clean)
//   * Determinism note: don’t assert JSON string ordering; assert loaded map equivalence
// * **What skill it builds for the project**
//
//   * CP2 persistence + failure mode handling (corrupt file)

use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    ptr::read,
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MyError {
    #[error("IO Error at {path}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("Invalid JSON in snapshot at {path}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("Failed to serialize JSON snapshot for {path}")]
    Serialize {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

type Map = HashMap<String, String>;

pub fn save(path: &Path, map: Map) -> Result<(), MyError> {
    let file = File::create(path).map_err(|e| MyError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    let mut writer = BufWriter::new(file);

    serde_json::to_writer(&mut writer, &map).map_err(|e| MyError::Serialize {
        path: path.to_path_buf(),
        source: e,
    })?;

    writer.flush().map_err(|e| MyError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    Ok(())
}

pub fn load(path: &Path) -> Result<Map, MyError> {
    let path = path.to_path_buf();
    let file = match File::open(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Map::new()),
        Err(error) => {
            return Err(MyError::Io {
                path,
                source: error,
            })
        }
    };

    let reader = BufReader::new(file);

    let map = serde_json::from_reader(reader).map_err(|e| MyError::Parse {
        path: path.to_path_buf(),
        source: e,
    })?;

    Ok(map)
}

#[cfg(test)]
mod mc2_tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_path(name: &str) -> PathBuf {
        let mut path = PathBuf::from("target");
        path.push(format!("test_{}", name));
        path
    }

    #[test]
    fn round_trip_save_then_load_equals_original() {
        let path = temp_path("round_trip.json");
        let _ = fs::remove_file(&path); // Clean up if exists

        let mut original = Map::new();
        original.insert("key1".to_string(), "value1".to_string());
        original.insert("key2".to_string(), "value2".to_string());
        original.insert("foo".to_string(), "bar".to_string());

        // Save the map
        save(&path, original.clone()).expect("save should succeed");

        // Load it back
        let loaded = load(&path).expect("load should succeed");

        // Assert equivalence (not JSON string ordering)
        assert_eq!(loaded, original);

        // Cleanup
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn load_nonexistent_file_returns_empty_map() {
        let path = temp_path("nonexistent.json");
        let _ = fs::remove_file(&path); // Ensure it doesn't exist

        let result = load(&path).expect("load should succeed for missing file");

        assert_eq!(result, Map::new());
        assert!(result.is_empty());
    }

    #[test]
    fn load_corrupt_json_returns_error() {
        let path = temp_path("corrupt.json");

        // Write invalid JSON
        fs::write(&path, b"{ this is not valid json }").expect("write should succeed");

        // Attempt to load
        let result = load(&path);

        assert!(result.is_err());
        match result {
            Err(MyError::Parse { path: _, source: _ }) => {
                // Expected error type
            }
            _ => panic!("Expected MyError::Parse, got {:?}", result),
        }

        // Cleanup
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn round_trip_with_empty_map() {
        let path = temp_path("empty.json");
        let _ = fs::remove_file(&path);

        let original = Map::new();

        save(&path, original.clone()).expect("save should succeed");
        let loaded = load(&path).expect("load should succeed");

        assert_eq!(loaded, original);
        assert!(loaded.is_empty());

        let _ = fs::remove_file(&path);
    }
}
