
# Pre-Project Day — REPL UX + Integration-Style Testing (stdin scripts, history, no-crash parsing)

## 1) Lecture Topic

* **Title:** REPL that survives humans: command parsing, history buffer, and “outside-in” tests
* **Why this matters for the project**

  * CP3 requires an interactive CLI that **does not crash** on empty input, extra spaces, wrong args, or unknown commands.
  * Integration tests (spawning a binary, feeding stdin) catch the real bugs unit tests miss: prompt loops, newline handling, and error messaging.
  * Command history (`VecDeque`) is a small feature that forces you to practice ownership + UX decisions without touching the engine.
* **Key concepts**

  * Parsing: `split_whitespace` + command enum + arity validation (no indexing panics)
  * UX contract: consistent responses (`ERR ...` vs `OK ...`), predictable newline behavior
  * REPL loop hygiene: read line, trim, handle empty, handle exit, never infinite-spin on EOF
  * History buffer: `VecDeque<String>` capped to last N commands (push + pop_front)
  * Determinism: stable output for tests (avoid timestamps; normalize whitespace)
  * Observability: errors to stderr vs responses to stdout (don’t mix in tests)
  * Integration-style tests: “scripted sessions” via `std::process::Command` (or `assert_cmd` later)
* **Tiny demo (≤10 lines)**

  ```rust
  use std::collections::VecDeque;
  fn push_hist(h: &mut VecDeque<String>, s: String, cap: usize) {
      h.push_back(s);
      while h.len() > cap { h.pop_front(); }
  }
  ```
* **“Prove you learned it” checklist**

  * You can implement a parser that returns typed `ParseError` for empty/unknown/wrong-arity.
  * You can implement command history (cap = 5) with correct eviction behavior.
  * You can write an integration-style test that feeds multiple lines and asserts stdout exactly.

---

## 2) Quiz (5 Questions)

**Q1) Why is `split_whitespace()` usually better than `split(' ')` for REPL parsing?**

* Tags: Concept + Difficulty 2 + Topics: REPL, Errors
* Answer key:

  * Collapses multiple spaces/tabs into a clean token stream
  * Avoids empty tokens from repeated spaces
  * Produces more robust parsing with less special-casing

**Q2) Reasoning: Your REPL returns user-facing messages. Where should “protocol” formatting live (e.g., `OK ...`, `ERR ...`), and what breaks if it leaks into core logic?**

* Tags: Reasoning + Difficulty 4 + Topics: Architecture, Testing, REPL
* Answer key:

  * Protocol formatting belongs at the CLI boundary (shell/binary side)
  * Core logic should return structured data / typed errors
  * If formatting leaks into core logic: harder to test, harder to reuse engine, coupling increases

**Q3) Bug-spotting: what can go wrong with this REPL loop, and how do you fix it?**

```rust
loop {
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    let line = line.trim();
    if line == "exit" { break; }
}
```

* Tags: Bug-Spotting + Difficulty 3 + Topics: REPL, Errors
* Answer key:

  * `unwrap()` panics (violates no-crash mindset)
  * Ignores EOF: `read_line` returns `Ok(0)` and you may spin forever
  * Fix: handle `Result`, break on `Ok(0)`, treat empty input as a no-op

**Q4) Tradeoff: Why keep REPL outputs deterministic (no timestamps, stable wording), even if “real systems” often log timestamps?**

* Tags: Tradeoff + Difficulty 3 + Topics: Testing, Observability, Determinism
* Answer key:

  * Deterministic stdout enables exact-match integration tests
  * Timestamps belong in logs (stderr) or structured logging layers, not in user protocol
  * Keeps behavior stable across machines/CI runs

**Q5) Reasoning: For a “last 5 commands” history, why is `VecDeque` a better fit than `Vec`, and what is the main cost?**

* Tags: Reasoning + Difficulty 3 + Topics: Collections, REPL, Performance
* Answer key:

  * `VecDeque` supports efficient pop/push at both ends (ideal for eviction)
  * `Vec` eviction at front is O(n) due to shifting
  * Cost: slightly more complex structure; not contiguous like `Vec` (minor overhead)

---

## 3) Mini-Challenges (3)

### Mini-Challenge 1

* **Name:** Parser v2: strict arity + normalized commands
* **Goal:** Build a robust command parser that is “REPL ready” and fully test-driven.
* **Setup (scratch folder)**

  * `cargo new mc3_1_parser && cd mc3_1_parser`
  * Implement logic in `src/lib.rs`
* **Requirements**

  * Define `enum Command { Set{key:String,val:String}, Get{key:String}, Del{key:String}, List, Exit, History }`
  * Define `enum ParseError { Empty, Unknown(String), WrongArity{ cmd:String, expected:&'static str } }`
  * Implement `fn parse(line: &str) -> Result<Command, ParseError>`
  * Command rules:

    * Case-insensitive commands accepted (`set`, `SET`, `SeT`)
    * Reject extra args for `GET/DEL/LIST/EXIT/HISTORY` (typed error)
  * Add **8 unit tests** covering: empty, unknown, each valid command, wrong arity, extra spaces, case-insensitive.
* **Proof**

  * `cargo test` passes with 8 parser tests
* **Guardrails**

  * No `unwrap/expect` in `src/lib.rs`
  * Deterministic errors (don’t depend on OS messages)
* **What skill it builds for the project**

  * CP3 “CLI survives invalid user input” requirement

---

### Mini-Challenge 2

* **Name:** History buffer with eviction + tests (VecDeque muscle memory)
* **Goal:** Practice `VecDeque` ownership + capping behavior exactly like the project’s REPL history.
* **Setup (scratch folder)**

  * `cargo new mc3_2_history && cd mc3_2_history`
  * Implement in `src/lib.rs`
* **Requirements**

  * Implement `struct History { cap: usize, buf: std::collections::VecDeque<String> }`
  * Methods:

    * `fn new(cap: usize) -> Self`
    * `fn push(&mut self, line: &str)` (stores an owned copy)
    * `fn items(&self) -> Vec<String>` (returns items oldest→newest)
  * Add tests:

    * pushing fewer than cap keeps all
    * pushing more than cap evicts oldest
    * ordering is correct and deterministic
* **Proof**

  * `cargo test` passes (3+ tests)
* **Guardrails**

  * No `unwrap/expect` in `src/lib.rs`
  * Deterministic order and stable output for tests
* **What skill it builds for the project**

  * CP3 “command history” should requirement + collections practice

---

### Mini-Challenge 3

* **Name:** Integration-style scripted REPL session (stdin → stdout contract)
* **Goal:** Write an “outside-in” test that simulates a user session, without any KV engine.
* **Setup (scratch folder)**

  * `cargo new mc3_3_repl_integ && cd mc3_3_repl_integ`
  * `src/main.rs` contains a minimal REPL loop that:

    * reads lines
    * uses your own parser (can be copied or reimplemented simply)
    * prints either `OK <cmd>` or `ERR <kind>`
    * supports `exit`
  * Add a test file: `tests/repl_session.rs`
* **Requirements**

  * REPL behavior:

    * Empty line → prints nothing (or prints `ERR Empty`) — choose one and test it
    * Unknown command → `ERR Unknown`
    * Wrong arity → `ERR Arity`
    * `exit` ends the loop
  * Integration-style test must:

    * Spawn the binary with `std::process::Command`
    * Pipe stdin with a multi-line script:

      * `SET a b`
      * `GET a`
      * `BOGUS`
      * `SET onlykey`
      * `exit`
    * Assert stdout contains the expected sequence (exact lines)
* **Proof**

  * `cargo test` runs the binary and passes the session test
* **Guardrails**

  * Keep outputs deterministic (no prompts like `> ` unless you include them in the expected output)
  * Put any debugging logs on stderr, not stdout
* **What skill it builds for the project**

  * CP3 integration testing + REPL loop correctness under real IO

---
