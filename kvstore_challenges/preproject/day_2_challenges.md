# Pre-Project Day — Persistence + Typed Errors (First-Run, Corruption, Determinism)

## 1) Lecture Topic

* **Title:** Crash-safe snapshot persistence: `Result` plumbing, “first run” semantics, and corrupt-file recovery
* **Why this matters for the project**

  * CP2 requires you to **load on startup** and **save on exit** without panics, including “file not found” on first run.
  * Corrupt data is a real failure mode; your store must fail gracefully and explain what happened.
  * Determinism + tests are easier if you design persistence boundaries cleanly (serialize/deserialize behind 2–3 functions).
* **Key concepts**

  * Error taxonomy: `Io`, `Serde`, and a semantic “CorruptData” vs “MissingFile”
  * Converting lower-level errors: `map_err`, `From`, and `thiserror` patterns (even if you don’t use the crate yet)
  * First-run behavior: treat `NotFound` as “empty DB” (not an error)
  * “No panics in lib”: avoid `read_to_string().unwrap()` and avoid indexing assumptions
  * Determinism note: don’t assert JSON ordering; assert round-trip equivalence and/or stable key listing
  * Observability: structured-ish stderr logs from the binary boundary (not the library)
  * Testing persistence: temp dirs, golden files, and corruption tests
* **Tiny demo (≤10 lines)**

  ```rust
  fn load_or_empty(path: &std::path::Path) -> Result<String, std::io::Error> {
      match std::fs::read_to_string(path) {
          Ok(s) => Ok(s),
          Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
          Err(e) => Err(e),
      }
  }
  ```
* **“Prove you learned it” checklist**

  * You can treat “missing file” as empty state while still surfacing other I/O errors.
  * You can distinguish “invalid JSON” from I/O failure using a typed error enum.
  * You can write a test that simulates corruption and verifies a non-panicking failure.

---

## 2) Quiz (5 Questions)

**Q1) In a persistence `load()` function, which I/O error kind should be treated as “first run = empty DB,” and why is it safe to special-case it?**

* Tags: Concept + Difficulty 2 + Topics: Persistence, Errors
* Answer key:

  * `std::io::ErrorKind::NotFound`
  * On first run, the DB file hasn’t been created yet
  * Other error kinds (PermissionDenied, InvalidData, etc.) are not “normal” and should surface

**Q2) You deserialize JSON into a `HashMap`. Parsing fails. Should that be an `Io` error or a `Serde`/Parse error—and what does that choice buy you?**

* Tags: Reasoning + Difficulty 3 + Topics: Serde, Errors, Persistence
* Answer key:

  * It’s a parse/serde error, not I/O
  * Separating categories makes RUNBOOK recovery actionable (“file corrupt” vs “disk problem”)
  * Enables better user messaging and better testing (assert error variant)

**Q3) Bug-spotting: why is this “load” implementation fragile, and what’s the safer pattern?**

```rust
fn load(path: &str) -> String {
    std::fs::read_to_string(path).unwrap()
}
```

* Tags: Bug-Spotting + Difficulty 2 + Topics: Persistence, Errors
* Answer key:

  * `unwrap()` panics (violates zero-panic library constraint)
  * Doesn’t handle first run (missing file)
  * Safer: return `Result<String, MyError>` and special-case `ErrorKind::NotFound` → empty

**Q4) Tradeoff: If JSON key order is unstable, what should your tests assert so they remain deterministic?**

* Tags: Tradeoff + Difficulty 3 + Topics: Testing, Determinism, Serde
* Answer key:

  * Assert round-trip equivalence of the in-memory map after `save`→`load`
  * Avoid asserting exact JSON text (ordering varies)
  * If you need stable text output, sort keys at display boundary or use a deterministic map/serializer strategy

**Q5) Reasoning: Where should stderr logging live—inside your persistence functions or at the call site—and what’s the failure mode if you put it in the wrong place?**

* Tags: Reasoning + Difficulty 4 + Topics: Architecture, Observability, Testing
* Answer key:

  * Prefer logging at the boundary (binary / shell), not inside pure library logic
  * Keeps library deterministic and test-friendly; avoids mixing I/O with core logic
  * Logging inside lib can cause noisy tests and hidden performance costs

---

## 3) Mini-Challenges (3)

### Mini-Challenge 1

* **Name:** Error enum drill: categorize I/O vs parse vs semantic corruption
* **Goal:** Build a `MyError` enum that mirrors what your KV project will need, without using the project itself.
* **Setup (scratch folder)**

  * `cargo new mc2_1_errors && cd mc2_1_errors`
  * Create `src/lib.rs`
* **Requirements**

  * Define:

    * `enum MyError { Io(std::io::Error), Parse(String), CorruptData { reason: String } }`
  * Implement `impl From<std::io::Error> for MyError`
  * Write 2 small functions in `lib.rs`:

    * `fn io_fail() -> Result<(), MyError>` that triggers a real I/O error (e.g., open a directory as a file OR open a clearly invalid path on your OS)
    * `fn parse_fail() -> Result<(), MyError>` that returns `Err(MyError::Parse(...))`
  * Add unit tests that assert:

    * `io_fail()` returns `Err(MyError::Io(_))`
    * `parse_fail()` returns `Err(MyError::Parse(_))`
* **Proof**

  * `cargo test` passes with both variant-assertion tests
* **Guardrails**

  * No `unwrap/expect` in `src/lib.rs`
  * Errors must be typed (no `String`-only `Result`)
* **What skill it builds for the project**

  * CP2 error taxonomy and conversion patterns

---

### Mini-Challenge 2

* **Name:** `load_or_empty` function with first-run semantics + tests
* **Goal:** Nail the single most common CP2 failure: treating missing file as a normal empty state.
* **Setup (scratch folder)**

  * `cargo new mc2_2_first_run && cd mc2_2_first_run`
  * Implement in `src/lib.rs`
* **Requirements**

  * Implement `fn load_or_empty(path: &std::path::Path) -> Result<String, MyError>`

    * If file missing: return `Ok(String::new())`
    * If readable: return contents
    * For other I/O errors: return `Err(MyError::Io(...))`
  * Add tests:

    * “missing file returns empty string”
    * “existing file returns exact content” (write file inside test)
* **Proof**

  * `cargo test` passes and shows both tests
* **Guardrails**

  * No `unwrap/expect` in `src/lib.rs` (in tests, use safe patterns too if you can)
  * Deterministic assertions (exact string compare)
* **What skill it builds for the project**

  * CP2 load behavior and reliability

---

### Mini-Challenge 3

* **Name:** Corruption simulation: distinguish invalid JSON from empty DB
* **Goal:** Practice the “corrupt file” path you’ll document in RUNBOOK (delete/repair file) without panicking.
* **Setup (scratch folder)**

  * `cargo new mc2_3_corrupt && cd mc2_3_corrupt`
  * Add deps: `serde`, `serde_json`
* **Requirements**

  * Use `type Map = std::collections::HashMap<String,String>;`
  * Implement:

    * `fn parse_map(json: &str) -> Result<Map, MyError>`

      * empty string → `Ok(empty_map)` (treat like no data)
      * invalid JSON → `Err(MyError::CorruptData { reason: ... })` OR `Err(MyError::Parse(...))` (choose one and be consistent)
  * Add tests:

    * `""` returns empty map
    * `"{"` returns the corruption/parse error variant (no panics)
    * valid JSON returns expected map
* **Proof**

  * `cargo test` passes with 3 parser tests
* **Guardrails**

  * No `unwrap/expect` in `src/lib.rs`
  * Deterministic error messages (don’t assert the entire serde error string unless you want brittleness)
* **What skill it builds for the project**

  * CP2 corrupt-file handling + RUNBOOK mindset

---
