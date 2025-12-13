// ## 3) Mini-Challenges (3)
//
// ### Mini-Challenge 1
//
// * **Name:** Error enum drill: categorize I/O vs parse vs semantic corruption
// * **Goal:** Build a `MyError` enum that mirrors what your KV project will need, without using the project itself.
// * **Setup (scratch folder)**
//
//   * `cargo new mc2_1_errors && cd mc2_1_errors`
//   * Create `src/lib.rs`
// * **Requirements**
//
//   * Define:
//
//     * `enum MyError { Io(std::io::Error), Parse(String), CorruptData { reason: String } }`
//   * Implement `impl From<std::io::Error> for MyError`
//   * Write 2 small functions in `lib.rs`:
//
//     * `fn io_fail() -> Result<(), MyError>` that triggers a real I/O error (e.g., open a directory as a file OR open a clearly invalid path on your OS)
//     * `fn parse_fail() -> Result<(), MyError>` that returns `Err(MyError::Parse(...))`
//   * Add unit tests that assert:
//
//     * `io_fail()` returns `Err(MyError::Io(_))`
//     * `parse_fail()` returns `Err(MyError::Parse(_))`
// * **Proof**
//
//   * `cargo test` passes with both variant-assertion tests
// * **Guardrails**
//
//   * No `unwrap/expect` in `src/lib.rs`
//   * Errors must be typed (no `String`-only `Result`)
// * **What skill it builds for the project**
//
//   * CP2 error taxonomy and conversion patterns

use std::{fs::File, io};

#[derive(Debug)]
enum KvError {
    Io(std::io::Error),
    Parse(String),
    InvalidData { reason: String },
}

impl From<std::io::Error> for KvError {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::InvalidData => KvError::InvalidData {
                reason: error.to_string(),
            },
            _ => KvError::Io(error),
        }
    }
}

impl From<serde_json::Error> for KvError {
    fn from(error: serde_json::Error) -> Self {
        return KvError::Parse(error.to_string());
    }
}

fn io_fail() -> Result<(), KvError> {
    let fail = File::open("fake path")?;
    Ok(())
}

fn parse_fail() -> Result<(), KvError> {
    let value = "{ invalid json }";
    let _fail: serde_json::Value = serde_json::from_str(value)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_fail_returns_io_error() {
        match io_fail() {
            Err(KvError::Io(_)) => {}
            other => panic!("expected Err(KvError::Io(_), got: {other:?}"),
        }
    }

    #[test]
    fn parse_fail_returns_parse_error() {
        match parse_fail() {
            Err(KvError::Parse(_)) => {}
            other => panic!("expected Err(KvError::Io(_), got: {other:?}"),
        }
    }
}

