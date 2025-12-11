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
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
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

pub fn load(path: &Path, map: Map) -> Result<Map, MyError> {
    Ok(map)
}
