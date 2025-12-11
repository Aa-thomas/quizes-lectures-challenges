# Pre-Project Day — KV Store Building Blocks Bootcamp (HashMap + Result + REPL Parse)

## 1) Lecture Topic

* **Title:** HashMap CRUD without panics: Option/Result flow + command parsing foundations
* **Why this matters for the project**

  * Your KV engine is mostly “HashMap semantics + ownership rules + no-crash error flow.”
  * Your REPL lives or dies on safe parsing (empty input, missing args, extra args) without panics.
  * Persistence later is just “serialize/deserialize a map,” which depends on clean error propagation today.
* **Key concepts**

  * `HashMap<String, String>`: ownership on insert/overwrite; borrowing on read (`Option<&V>`)
  * Why `get` returns `Option<&V>` and how to convert that into a user-friendly `Result`
  * Typed error enums vs stringly errors (what breaks in prod)
  * “Library-style code” rules: no `unwrap/expect`, use `?`, return `Result`
  * Determinism note: stable outputs for tests (sort keys before printing)
  * REPL parsing basics: `split_whitespace`, command enum, arity validation
  * Observability basics: log I/O-ish actions to stderr (even in drills)
* **Tiny demo (≤10 lines)**

  ```rust
  fn parse(line: &str) -> Option<(&str, Vec<&str>)> {
      let mut it = line.split_whitespace();
      let cmd = it.next()?;
      Some((cmd, it.collect()))
  }
  ```
* **“Prove you learned it” checklist**

  * You can write a function that converts `Option<&String>` into `Result<String, MyError>` without cloning unnecessarily.
  * You can parse commands safely (including empty input) and validate argument counts with tests.
  * You can print a deterministic “list” output by sorting keys before display.

---

## 2) Quiz (5 Questions)

**Q1) Why does `HashMap::get` return `Option<&V>` instead of `V`?**
My Answer  ``` It returns option because the key may or may not exist, there for it needs a Some value and a None value. It returns a reference because the hashmap is the owner of the value and get just allows you to READ the value.```
```
```
```
```
* Tags: Concept + Difficulty 2 + Topics: Ownership, Collections
* Answer key:

  * Key may not exist → `None`
  * Returning `&V` avoids moving/cloning; map still owns the value
  * Returning `V` would require moving out (not allowed) or cloning (costly)

**Q2) You want a “library-style” function `fn read_key(...) -> Result<String, E>`. You currently have `Option<&String>` from `get`. What’s the cleanest conversion strategy and why?**
*My Answer* - ``` Clone and then use ok_or else. One option is to use (functionGoesHere).map(|foo| foo.cloned().ok_or_else(Err(MyErrorGoesHere) or you can use a simpler syntax by writing (functionGoesHere).cloned.ok_or_else(Err(MyErrorGoesHere)). The key is you need clone the value and then use it with ok_or() or ok_```
* Tags: Reasoning + Difficulty 3 + Topics: Errors, Ownership, Collections
* Answer key:

  * Use `ok_or_else(...)` / `ok_or(...)` to convert `Option` → `Result`
  * Return a typed “not found” error variant (not a string)
  * Clone only at the boundary if you must return an owned `String` (explain tradeoff)

**Q3) Bug-spotting: what’s wrong with this “no panics” helper, and how do you fix it?**

* My Answer * - ```This helper doesnt do any bounds checking to make sure that parts[0] and parts[1] will exist after the split. Splitting by ' ' can fail and the line can be empty. It should return an option enum where it returns None in this case. split_whitespace does a better job of splitting whitespace because it handles tabs, newlines,etc not just space.  ``` 
```

```rust
fn must_parse(line: &str) -> (&str, &str) {
    let parts: Vec<&str> = line.split(' ').collect();
    (parts[0], parts[1])
}
```

* Tags: Bug-Spotting + Difficulty 3 + Topics: Errors, REPL, Ownership
* Answer key:

  * Indexing can panic (`parts[0]`, `parts[1]`) on empty/short input
  * Splitting by `' '` mishandles multiple spaces; use `split_whitespace`
  * Fix by returning `Option`/`Result` and validating arity (or pattern-match on iterator)

**Q4) Tradeoff: In a “LIST keys” command, why might you sort keys before printing even if HashMap is faster than BTreeMap?**
* My Answer * - ``` Hashmaps do not maintain the order that they were inserted. You need to sort it for deterministic testing. When iterating over a hashmap this matters because you can get a different order each time. You should always sort before displaying keys from a hashmap. This makes it easy to debug because they same inputs produce the same outputs.```

* Tags: Tradeoff + Difficulty 3 + Topics: Collections, Testing, Determinism
* Answer key:

  * HashMap iteration order is not stable → tests/flaky output
  * Sorting gives deterministic output for snapshots and integration tests
  * Keep HashMap for O(1) access; sort only at presentation boundary

**Q5) Reasoning: For persistence, you’ll serialize/deserialize a map. What two distinct error categories should exist in a typed error enum, and what should they carry?**

* (Failed) My Answer - ``` I couldnt answer on my own, but after looking at notes, the two error categories are IO errors and Parsing/Serialization Errors```

* Tags: Reasoning + Difficulty 4 + Topics: Errors, Serde, Persistence
* Answer key:

  * I/O errors (wrap `std::io::Error` or store message/context)
  * Serialization/parse errors (wrap serde error or store message/context)
  * Optional: semantic error like “corrupt data” vs “missing file” (first run)

---

## 3) Mini-Challenges (3)

### Mini-Challenge 1

* **Name:** Typed “Not Found” without panics (Option → Result drill)
* **Goal:** Practice the exact error-flow pattern you’ll use in the engine: “missing key becomes a typed error,” not a crash.
* **Setup (scratch folder)**

  * `cargo new mc1_not_found && cd mc1_not_found`
  * Create `src/lib.rs` and `src/main.rs` (main can be tiny).
* **Requirements**

  * Define `enum MyError { NotFound { key: String } }`
  * Implement `fn get_owned(map: &std::collections::HashMap<String,String>, key: &str) -> Result<String, MyError>`
  * Must not use `unwrap/expect` anywhere in `src/lib.rs`
  * Add **2 unit tests**:

    * returns `Ok(value)` for existing key
    * returns `Err(MyError::NotFound{...})` for missing key
* **Proof**

  * `cargo test` shows 2 passing tests
* **Guardrails**

  * No `unwrap/expect` in `src/lib.rs`
  * Deterministic tests (assert exact error variant contents)
* **What skill it builds for the project**

  * Engine correctness + typed errors (CP1 → CP2 readiness)

---

### Mini-Challenge 2

* **Name:** JSON snapshot round-trip (map ↔ disk) with error propagation
* **Goal:** Build the muscle memory for persistence (save/load) without touching the real repo.
* **Setup (scratch folder)**

  * `cargo new mc2_snapshot && cd mc2_snapshot`
  * Add deps: `serde`, `serde_json` (derive enabled)
* **Requirements**

  * Use `type Map = std::collections::HashMap<String,String>;`
  * Implement in `src/lib.rs`:

    * `fn save(path: &std::path::Path, map: &Map) -> Result<(), MyError>`
    * `fn load(path: &std::path::Path) -> Result<Map, MyError>`
  * Behavior rules:

    * If file does not exist on load: return `Ok(empty_map)` (first-run behavior)
    * If JSON is invalid: return a typed error (no panics)
  * Add tests using a temp directory (or write to `target/` with a unique filename):

    * round-trip: save then load equals original map
    * corrupt file: write invalid JSON then `load` returns `Err(...)`
* **Proof**

  * `cargo test` passes and shows both persistence tests
* **Guardrails**

  * No `unwrap/expect` in `src/lib.rs`
  * Log “save/load attempted” to stderr from `main.rs` only (keep lib clean)
  * Determinism note: don’t assert JSON string ordering; assert loaded map equivalence
* **What skill it builds for the project**

  * CP2 persistence + failure mode handling (corrupt file)

---

### Mini-Challenge 3

* **Name:** REPL command parser with arity validation (no crashes)
* **Goal:** Practice parsing commands like `SET k v`, `GET k`, `DEL k`, `LIST` safely and predictably.
* **Setup (scratch folder)**

  * `cargo new mc3_repl_parse && cd mc3_repl_parse`
  * Create `src/lib.rs` with parsing logic + `src/main.rs` that just reads a line and prints parsed result.
* **Requirements**

  * Define:

    * `enum Command { Set { key: String, val: String }, Get { key: String }, Del { key: String }, List, Exit }`
    * `enum ParseError { Empty, UnknownCmd(String), WrongArity { cmd: String, expected: &'static str } }`
  * Implement `fn parse_command(line: &str) -> Result<Command, ParseError>`
  * Must handle:

    * empty line
    * extra spaces (use `split_whitespace`)
    * wrong arg counts (typed error)
    * unknown command
  * Add **unit tests** for at least 6 cases:

    * valid `SET a b`
    * valid `GET a`
    * empty input
    * unknown cmd
    * wrong arity for `SET`
    * `LIST` with extra args rejected (or explicitly decide and test it)
* **Proof**

  * `cargo test` passes with 6+ parser tests
  * `cargo run` allows typing a line and prints `Ok(...)` or `Err(...)` deterministically
* **Guardrails**

  * No `unwrap/expect` in `src/lib.rs`
  * Deterministic display (derive `Debug` and print via `println!("{:?}", ...)`)
* **What skill it builds for the project**

  * CP3 REPL robustness (survives invalid input)

---
