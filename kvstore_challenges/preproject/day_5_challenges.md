
# Pre-Project Day — Polish Mindset: Clippy-Strict, Docs, Runbook Thinking, and “No-Panic” Discipline

## 1) Lecture Topic

* **Title:** Production readiness habits: zero-panics, clippy -D warnings, actionable docs, and failure-mode playbooks
* **Why this matters for the project**

  * CP4 is where most “almost done” projects fail: lingering panics, weak docs, messy errors, and unverifiable behavior.
  * Hiring signal artifacts (`README/DESIGN/BENCH/RUNBOOK/tests/demo`) are proofs, not vibes—your habits must produce evidence.
  * Real systems don’t just “work”; they fail predictably, explain themselves, and are easy to operate.
* **Key concepts**

  * “No unwrap/expect in library code” as an invariant; panics are latent outages
  * Clippy strictness as a feedback loop: fix root causes, not symptoms
  * Error message quality: distinguish user error vs system error; stable wording for tests
  * RUNBOOK mindset: predictable recovery steps (corrupt file, permission denied, bad input)
  * README mindset: minimal commands + examples + expected outputs
  * DESIGN mindset: justify tradeoffs (HashMap vs BTreeMap, snapshot vs WAL)
  * Demo script mindset: deterministic scripted session (stdin → stdout) you can re-run
* **Tiny demo (≤10 lines)**

  ```rust
  // Library-style: convert Option -> Result without panics
  fn require<T>(opt: Option<T>, msg: &'static str) -> Result<T, &'static str> {
      opt.ok_or(msg)
  }
  ```
* **“Prove you learned it” checklist**

  * You can run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` successfully in a scratch crate.
  * You can write a short RUNBOOK section that maps a failure to a recovery action.
  * You can create a deterministic “demo session” script and assert its output.

---

## 2) Quiz (5 Questions)

**Q1) What’s the difference between a “user error” and a “system error” in a CLI, and how should each be communicated?**

* Tags: Concept + Difficulty 3 + Topics: Errors, REPL, Documentation
* Answer key:

  * User error: bad command/arity/invalid input → friendly message + usage hint (stdout or structured “ERR”)
  * System error: I/O failure, permission denied, corrupt file → actionable message + recovery steps (stderr + RUNBOOK)
  * Keeping them distinct improves UX and operability

**Q2) Reasoning: Why is “no unwrap/expect in lib” more than style—what are two concrete failure modes it prevents in this KV project?**

* Tags: Reasoning + Difficulty 4 + Topics: Errors, Reliability, Persistence
* Answer key:

  * Missing key / empty input would crash instead of returning a controlled error
  * File not found / corrupt JSON would crash instead of first-run empty or recoverable failure
  * Panics make integration tests flaky and invalidate reliability claims

**Q3) Bug-spotting: what’s wrong with this error handling pattern for persistence, from an operability perspective?**

```rust
return Err(MyError::Parse("bad file".to_string()));
```

* Tags: Bug-Spotting + Difficulty 3 + Topics: Errors, Persistence, Runbook
* Answer key:

  * Too vague: no context (which file? what happened?)
  * Doesn’t differentiate “corrupt file” vs “invalid format” vs “empty file”
  * Better: include path/context and map to a recovery action (e.g., “delete db file”)

**Q4) Tradeoff: In docs, why is a short, deterministic “demo script” often more valuable than a long explanation?**

* Tags: Tradeoff + Difficulty 3 + Topics: Testing, Documentation, REPL
* Answer key:

  * It’s executable proof: anyone can rerun and verify behavior
  * Prevents ambiguity and drift between docs and reality
  * Helps during interviews (“here’s exactly what it does”)

**Q5) Reasoning: If `cargo clippy -- -D warnings` fails, what is the best workflow to fix issues without cargo-culting?**

* Tags: Reasoning + Difficulty 4 + Topics: Testing, Architecture, Reliability
* Answer key:

  * Fix highest-risk issues first (panic paths, error handling, ownership mistakes)
  * Prefer refactors that reduce complexity (simpler control flow, fewer clones)
  * Add/adjust tests to lock in behavior before refactoring
  * Avoid blanket `#[allow]` except for well-justified cases (like tests)

---

## 3) Mini-Challenges (3)

### Mini-Challenge 1

* **Name:** Zero-panic audit drill: delete all unwraps (library-only) + tests
* **Goal:** Build the reflex: every panic path becomes a typed error or safe branch.
* **Setup (scratch folder)**

  * `cargo new mc5_1_no_panic && cd mc5_1_no_panic`
  * Put “library logic” in `src/lib.rs`
* **Requirements**

  * In `lib.rs`, implement:

    * `fn parse_u32(s: &str) -> Result<u32, MyError>` (no `.parse().unwrap()`)
    * `fn second_token(line: &str) -> Result<String, MyError>` (no indexing)
  * `MyError` must include at least:

    * `InvalidNumber`
    * `MissingToken`
  * Add tests:

    * `parse_u32("42")` ok
    * `parse_u32("nope")` returns `InvalidNumber`
    * `second_token("GET key")` ok
    * `second_token("GET")` returns `MissingToken`
* **Proof**

  * `cargo test` passes
  * `cargo clippy -- -D warnings` passes (aim for clean; if a warning appears, fix it)
* **Guardrails**

  * No `unwrap/expect` in `src/lib.rs`
  * Deterministic errors (stable variants)
* **What skill it builds for the project**

  * CP4 “zero panics” + edge-case correctness under strict linting

---

### Mini-Challenge 2

* **Name:** Runbook simulator: map failures → recovery actions (executable)
* **Goal:** Practice writing *actionable* recovery guidance backed by behavior (not just prose).
* **Setup (scratch folder)**

  * `cargo new mc5_2_runbook_sim && cd mc5_2_runbook_sim`
  * `src/lib.rs` returns structured failures; `src/main.rs` prints actions
* **Requirements**

  * In `lib.rs`, define:

    * `enum Failure { MissingFile, CorruptFile, PermissionDenied }`
    * `fn classify_io(e: &std::io::Error) -> Failure` (based on `ErrorKind` when possible)
    * `fn recovery(f: Failure) -> &'static str` returning a one-line action, e.g.:

      * MissingFile → “Create empty DB and continue”
      * CorruptFile → “Move db file aside and restart”
      * PermissionDenied → “Check file permissions / run in writable directory”
  * In `main.rs`, simulate each failure and print:

    * `FAIL: <kind>\nACTION: <recovery>`
  * Add tests:

    * `recovery(CorruptFile)` returns the expected action line
* **Proof**

  * `cargo test` passes
  * `cargo run` prints deterministic “FAIL/ACTION” blocks
* **Guardrails**

  * Keep I/O and printing in `main.rs`
  * No unwrap/expect in `lib.rs`
* **What skill it builds for the project**

  * RUNBOOK quality + CP2/CP4 operability mindset

---

### Mini-Challenge 3

* **Name:** Demo script contract: deterministic REPL transcript (golden output)
* **Goal:** Create the “demo/” muscle memory: a scriptable session with exact expected output.
* **Setup (scratch folder)**

  * `cargo new mc5_3_demo_contract && cd mc5_3_demo_contract`
  * `src/main.rs` implements a minimal command loop (no KV engine needed):

    * Accepts `PING`, `ECHO <msg>`, `EXIT`
    * On invalid input prints `ERR`
  * Create a test: `tests/transcript.rs`
* **Requirements**

  * Integration-style test must:

    * Spawn the binary
    * Feed stdin:

      * `PING`
      * `ECHO hello`
      * `BOGUS`
      * `EXIT`
    * Assert stdout equals exactly:

      * `PONG`
      * `hello`
      * `ERR`
  * Keep prompts out of stdout (or include them in expected output—your choice, but deterministic)
* **Proof**

  * `cargo test` passes (integration test runs the binary)
* **Guardrails**

  * Deterministic stdout only (no timestamps, no random data)
  * Any debug info goes to stderr
* **What skill it builds for the project**

  * CP3/CP4 demo artifact + integration test discipline for the real REPL

---
