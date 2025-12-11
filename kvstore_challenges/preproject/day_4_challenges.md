
# Pre-Project Day — Benchmarking + Determinism + Clean Architecture Boundaries

## 1) Lecture Topic

* **Title:** Measure before you optimize: Criterion baselines + deterministic measurement + “engine vs shell” separation
* **Why this matters for the project**

  * CP3/CP4 require you to produce a credible `BENCH.md` and avoid “benching println! instead of logic.”
  * Your engine must remain reusable and testable; mixing REPL/persistence/logging into core logic makes everything harder to measure and reason about.
  * Deterministic outputs and clean boundaries prevent flaky tests and misleading performance claims.
* **Key concepts**

  * What to benchmark: pure operations (insert/get/remove) isolated from I/O and formatting
  * Criterion basics: warm-up, sample size, what results mean (variance, noise)
  * Avoiding measurement traps: allocations, hashing, debug printing, RNG seeds
  * “Functional core / imperative shell” in practice: core functions are pure-ish; shell handles stdin/stdout/fs/logging
  * Determinism in user-facing outputs: stable key listing (sort keys) and stable error messages
  * Big-O vs constants: Hashing + allocation costs dominate small workloads
  * Minimal observability rule: stderr logs at boundaries, not inside hot paths
* **Tiny demo (≤10 lines)**

  ```rust
  fn hot_get(map: &std::collections::HashMap<String,String>, k: &str) -> bool {
      map.get(k).is_some()
  }
  ```
* **“Prove you learned it” checklist**

  * You can write a Criterion bench that times a pure function (no I/O, no printing).
  * You can explain why `set` is often slower than `get` (hash + allocation + potential resize).
  * You can enforce a boundary rule: core logic returns data/errors; shell formats + logs.

---

## 2) Quiz (5 Questions)

**Q1) What’s the biggest mistake beginners make when benchmarking CLI programs, and how do you avoid it?**

* Tags: Concept + Difficulty 2 + Topics: Bench, REPL, Performance
* Answer key:

  * Benchmarking printing/logging instead of core logic
  * Running benchmarks in debug mode
  * Fix: bench pure functions in a library module under `benches/` with release settings

**Q2) Reasoning: Why might a `set` operation be slower than `get` in a `HashMap<String, String>` even though both are “O(1)” on paper?**

* Tags: Reasoning + Difficulty 3 + Topics: Collections, Bench, Performance
* Answer key:

  * `set` may allocate for new `String` data (or clone/own it)
  * Hashing the key + potential table growth/rehash
  * `get` is read-only (no allocation), often fewer steps

**Q3) Bug-spotting: what’s flawed about this benchmark pattern?**

```rust
b.iter(|| {
    println!("{}", map.get("a").unwrap());
});
```

* Tags: Bug-Spotting + Difficulty 3 + Topics: Bench, Testing, Errors
* Answer key:

  * Includes `println!` cost (dominates runtime)
  * Uses `unwrap()` (panic risk, also not representative)
  * Fix: measure the operation result without printing; avoid unwrap by checking `is_some()` or using known-present key setup

**Q4) Tradeoff: If you need deterministic “LIST” output, do you switch to `BTreeMap` or sort keys on demand?**

* Tags: Tradeoff + Difficulty 4 + Topics: Collections, Determinism, Architecture
* Answer key:

  * Keep `HashMap` for O(1) access and sort keys on demand for display
  * `BTreeMap` gives ordered iteration but changes perf profile (O(log n))
  * Sorting on demand isolates determinism to UX boundary, not core storage choice

**Q5) Reasoning: Where should structured logs go (stderr) and why should they be excluded from benchmarks?**

* Tags: Reasoning + Difficulty 4 + Topics: Observability, Bench, Architecture
* Answer key:

  * Logs belong in shell/binary boundary; stderr for diagnostics
  * Logging adds I/O and synchronization overhead → ruins microbench validity
  * Keeps core functions pure-ish and predictable for testing + measurement

---

## 3) Mini-Challenges (3)

### Mini-Challenge 1

* **Name:** Criterion baseline: `insert` vs `get` (pure functions only)
* **Goal:** Produce a credible baseline benchmark that compares core operations without I/O noise.
* **Setup (scratch folder)**

  * `cargo new mc4_1_bench && cd mc4_1_bench`
  * Add dev-dep: `criterion`
  * Create `benches/basic.rs`
* **Requirements**

  * Implement in `src/lib.rs` two pure helpers:

    * `fn do_get(map: &HashMap<String,String>, key: &str) -> bool`
    * `fn do_set(map: &mut HashMap<String,String>, key: String, val: String)`
  * Criterion bench must:

    * Pre-build a map with N entries for the `get` bench
    * For `set` bench, avoid measuring setup each iteration (use patterns like cloning a small map per iter OR use `iter_batched` conceptually—without needing advanced APIs if you don’t want)
    * Print **nothing** during benchmarking
* **Proof**

  * `cargo bench` runs and shows timings for `get` and `set`
  * Write a short `BENCH_NOTES.md` (5–8 lines) summarizing which is slower and why
* **Guardrails**

  * No `unwrap/expect` in `src/lib.rs`
  * Deterministic benchmark inputs (fixed keys, fixed sizes)
* **What skill it builds for the project**

  * CP3/CP4 benchmarking credibility (`BENCH.md` readiness)

---

### Mini-Challenge 2

* **Name:** Deterministic list output (HashMap → sorted keys)
* **Goal:** Practice generating stable output from an unordered structure (prevents flaky CLI tests).
* **Setup (scratch folder)**

  * `cargo new mc4_2_determinism && cd mc4_2_determinism`
  * Implement in `src/lib.rs`
* **Requirements**

  * Implement `fn sorted_keys(map: &HashMap<String,String>) -> Vec<String>`

    * Returns keys sorted ascending
  * Add tests:

    * Insert keys in random order and assert the returned vector is sorted
    * Ensure it doesn’t mutate the map
  * Optional: Implement `fn format_list(keys: &[String]) -> String` that returns newline-separated output deterministically
* **Proof**

  * `cargo test` passes (2+ tests)
* **Guardrails**

  * No panics in `src/lib.rs` (no indexing assumptions)
  * Deterministic output formatting (normalize trailing newline decisions)
* **What skill it builds for the project**

  * CP3 REPL `list` UX + deterministic integration tests

---

### Mini-Challenge 3

* **Name:** Architecture boundary drill: pure core + I/O shell wrapper
* **Goal:** Build a tiny “core vs shell” separation habit without building the real KV store.
* **Setup (scratch folder)**

  * `cargo new mc4_3_boundaries && cd mc4_3_boundaries`
  * `src/lib.rs` = core logic
  * `src/main.rs` = shell (stdin/stdout/stderr)
* **Requirements**

  * In `lib.rs`, implement a pure-ish function:

    * `fn apply(op: &str, a: i32, b: i32) -> Result<i32, CoreError>`
    * Supported ops: `add`, `sub`, `mul`, `div`
    * Division by zero returns a typed error (no panic)
  * In `main.rs`:

    * Read one line like `add 2 3`
    * Parse input safely
    * Call `apply`
    * Print result to stdout; errors to stderr
  * Add tests in `lib.rs`:

    * `div 10 0` returns the right error variant
    * `mul 3 4` returns 12
* **Proof**

  * `cargo test` passes
  * `cargo run` with sample inputs prints deterministic output (`12` etc.)
* **Guardrails**

  * No `unwrap/expect` in `src/lib.rs`
  * Keep all I/O in `main.rs`
* **What skill it builds for the project**

  * CP1–CP4 architecture separation habit (lib vs bin), plus error handling under pressure
