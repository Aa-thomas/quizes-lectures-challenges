# Crash-Safe Snapshot Persistence: Result Plumbing, First-Run Semantics, and Corrupt-File Recovery

## Introduction: Why Your Data Store Can't Just "Hope for the Best"

Welcome to what might be the most important lecture you'll encounter when building real-world systems. Today we're going to talk about something that separates toy projects from production-ready software: crash-safe persistence. We're going to explore how to save and load data in a way that doesn't panic when things go wrong, and trust me, things will go wrong.

Think about this scenario for a moment. Your key-value store has been running beautifully for weeks. A user has stored hundreds of entries in it. Then one day, their laptop battery dies mid-write. Or their cat walks across the keyboard and corrupts the data file. Or they accidentally delete the file and restart your program. What happens? Does your program explode with an ugly panic message? Does it silently eat their data? Or does it gracefully handle the problem and give them useful feedback about what went wrong?

The difference between these outcomes is what we're going to master today. In CP2, you'll need to load your store on startup and save it on exit without any panics. That includes handling the very first run when no file exists yet, dealing with corrupted data, and providing clear error messages when something truly goes wrong. This isn't just about satisfying test requirements; it's about building software that respects your users and their data.

## Part 1: Understanding the Error Landscape

Before we dive into solutions, we need to understand the problem space. When you're reading and writing files, especially when serializing structured data, you're actually facing several distinct types of failures. Let's build a taxonomy of what can go wrong.

### The Three Categories of Persistence Errors

First, you have input/output errors, what we typically call I/O errors. These come from the operating system level. Maybe the disk is full. Maybe you don't have permission to write to that directory. Maybe the file path is invalid. Maybe, and this is crucial, the file simply doesn't exist yet. All of these manifest as `std::io::Error` in Rust.

Second, you have serialization and deserialization errors. Let's say you're using JSON or some other format to save your key-value store. The file might exist and be readable at the I/O level, but its contents might not be valid JSON. Or it might be valid JSON but not match the structure your code expects. Perhaps someone manually edited the file and introduced a syntax error, or maybe an old version of your program wrote it with a different schema. These are serialization errors, and if you're using serde_json, they show up as `serde_json::Error`.

Third, and this is where it gets interesting, you have semantic or domain-level errors. These are errors that are specific to your application logic. For instance, maybe the JSON is perfectly valid, and it deserializes into your data structures just fine, but the data itself is nonsensical. Perhaps there are duplicate keys where there shouldn't be, or negative values where only positive ones make sense. We might call this category "CorruptData" to distinguish it from the lower-level failures.

Now here's the key insight: these three categories need to be handled differently. A missing file on first run is not an error at all; it's expected behavior. But a missing file on the hundredth run might indicate that someone deleted your data file, and you should tell the user about it. Invalid JSON is different from a disk failure. The user needs to know which one happened so they can take appropriate action.

### Why Error Types Matter

You might be wondering why we can't just use string error messages everywhere. After all, we could just return `Result<T, String>` and call it a day, right? This works in very simple cases, but it falls apart quickly. The problem is that strings aren't composable and they're not programmatically distinguishable.

Imagine you have a function that loads your data, and it returns an error. Your calling code might want to handle different errors differently. If it's a "file not found" error on first run, you want to treat that as an empty store and continue happily. But if it's a permission error, you want to show the user an error message and exit. And if it's corrupted data, you might want to attempt recovery or ask the user what to do. With string errors, you'd have to parse the error message text to figure out what happened, which is fragile and error-prone.

Instead, we want typed errors that carry semantic meaning. In Rust, this typically means creating an enum that represents all the different failure modes your system can encounter.

## Part 2: Building a Typed Error System

Let's design an error type that captures the distinctions we've discussed. We'll start simple and build up complexity as we understand the needs better.

```rust
#[derive(Debug)]
pub enum PersistenceError {
    Io(std::io::Error),
    Serialization(String),
    CorruptData(String),
}
```

This enum says "a persistence operation can fail in three ways." We can wrap an I/O error, we can have a serialization problem with some description, or we can have semantically corrupt data with an explanation of what's wrong. Notice that we derive `Debug` so we can print these errors in a developer-friendly way.

Now, when you write your loading function, instead of returning `Result<YourData, std::io::Error>`, you return `Result<YourData, PersistenceError>`. This immediately makes your API more honest. It's telling callers "I might fail in these specific ways" rather than pretending it can only fail with I/O errors.

### The Magic of Error Conversion

But here's the problem: when you call `std::fs::read_to_string(path)`, it returns a `Result<String, std::io::Error>`. When you call `serde_json::from_str(data)`, it returns a `Result<YourType, serde_json::Error>`. These aren't your error type. How do you convert them?

The naive approach is to use `map_err` at every call site. You'd write something like:

```rust
let contents = std::fs::read_to_string(path)
    .map_err(|e| PersistenceError::Io(e))?;
let data = serde_json::from_str(&contents)
    .map_err(|e| PersistenceError::Serialization(e.to_string()))?;
```

This works, but it's verbose and repetitive. Rust gives us a better way through the `From` trait. If you implement `From<std::io::Error>` for your error type, then the question mark operator will automatically perform the conversion for you.

```rust
impl From<std::io::Error> for PersistenceError {
    fn from(error: std::io::Error) -> Self {
        PersistenceError::Io(error)
    }
}

impl From<serde_json::Error> for PersistenceError {
    fn from(error: serde_json::Error) -> Self {
        PersistenceError::Serialization(error.to_string())
    }
}
```

With these implementations in place, your code becomes much cleaner:

```rust
let contents = std::fs::read_to_string(path)?;
let data = serde_json::from_str(&contents)?;
```

The question mark operator sees that you're returning a `Result<_, PersistenceError>` from your function, and it automatically converts the lower-level errors using your `From` implementations. This is one of those features where Rust's type system really shines. You get automatic, type-safe error conversion with no runtime overhead.

### A Note on Thiserror

In real projects, you'll often see teams using the `thiserror` crate, which automates this boilerplate. You'd write something like:

```rust
#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Data corruption detected: {0}")]
    CorruptData(String),
}
```

The `#[from]` attribute automatically generates those `From` implementations we wrote manually, and the `#[error]` attribute provides nice display formatting. Even if you're not using this crate yet in your project, understanding the pattern helps you see what problem it's solving and how to implement it yourself.

## Part 3: First-Run Semantics and the "Missing File" Problem

Now we get to one of the trickiest parts of persistence design: handling the first run of your program. When your key-value store runs for the very first time, there is no data file. Should this be an error? The answer is emphatically no, but implementing this correctly requires some thought.

### The Pattern: Match on ErrorKind

Here's the core pattern you'll use throughout your persistence code:

```rust
fn load_or_empty(path: &std::path::Path) -> Result<String, std::io::Error> {
    match std::fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(e),
    }
}
```

Let me walk through this carefully because it embodies an important principle. We attempt to read the file. If that succeeds, great, we return the contents. If it fails, we don't just immediately propagate the error. Instead, we examine the error to see what kind it is. If it's specifically a `NotFound` error, we treat that as success and return an empty string, which represents our empty initial state. But if it's any other kind of I/O error—permission denied, disk error, path is a directory instead of a file, whatever—we propagate that as a genuine error.

This pattern respects the principle of treating expected conditions differently from exceptional conditions. A missing file on first run is expected. A permission error is exceptional and needs to be surfaced to the user.

### Integrating With Your Full Load Function

Your complete loading function might look something like this:

```rust
pub fn load_from_file(path: &std::path::Path) -> Result<Store, PersistenceError> {
    // First, try to read the file, treating NotFound as empty content
    let contents = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // First run: return an empty store
            return Ok(Store::new());
        }
        Err(e) => return Err(PersistenceError::Io(e)),
    };

    // If the file exists but is empty, also treat as empty store
    if contents.trim().is_empty() {
        return Ok(Store::new());
    }

    // Try to deserialize the JSON
    let data: Store = serde_json::from_str(&contents)
        .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

    // Optionally: validate the data structure makes sense
    // For example, check for duplicate keys, valid values, etc.
    
    Ok(data)
}
```

Notice the layered approach here. We handle file-not-found specially. We also handle empty files gracefully, which might occur if a previous save was interrupted. Then we try to deserialize. Each layer can fail in its own way, and we handle each appropriately.

## Part 4: The "No Panics in Library Code" Principle

This is a design principle that will serve you well throughout your programming career: library code should never panic. By "library code," I mean the reusable parts of your program, the parts that aren't directly handling user interaction. In your project structure, this typically means everything except your `main.rs` or binary entry points.

### Why This Matters

When library code panics, it takes away control from the calling code. The caller can't catch the panic in normal flow control. They can't decide how to handle the error. They can't log it properly. They certainly can't recover from it gracefully. Panics are for unrecoverable programmer errors or truly exceptional conditions, not for expected failure modes like "file not found" or "invalid data format."

### Common Panic Traps in Persistence Code

Let me show you some common patterns that students write, which seem to work but are actually ticking time bombs:

```rust
// DANGER: This panics if the file doesn't exist
let contents = std::fs::read_to_string(path).unwrap();

// DANGER: This panics if the JSON is invalid
let data: Store = serde_json::from_str(&contents).unwrap();

// DANGER: This panics if the key doesn't exist
let value = store.get("some_key").unwrap();

// DANGER: This panics if the vector is empty
let first = items[0];
```

Every one of these `unwrap()` calls is a potential panic. Every direct index operation is a potential panic. In production code, these are bugs waiting to happen. Instead, you should be using the question mark operator for `Result` types, using `if let` or `match` for `Option` types, and using safer indexing methods like `get()` that return `Option`.

### The Boundary Principle

So if library code shouldn't panic, where should you handle errors? The answer is at the boundary of your system, typically in your `main()` function or other binary entry points. This is where you translate technical errors into user-facing messages. This is where you decide whether to retry, exit gracefully, or continue with degraded functionality.

Your `main()` function might look like this:

```rust
fn main() {
    let store_path = get_store_path();
    
    let mut store = match load_from_file(&store_path) {
        Ok(s) => {
            eprintln!("Loaded {} entries from store", s.len());
            s
        }
        Err(PersistenceError::Io(e)) => {
            eprintln!("Error reading store file: {}", e);
            eprintln!("Starting with empty store");
            Store::new()
        }
        Err(PersistenceError::Serialization(e)) => {
            eprintln!("Error parsing store file: {}", e);
            eprintln!("The store file may be corrupted");
            eprintln!("Starting with empty store");
            Store::new()
        }
        Err(PersistenceError::CorruptData(e)) => {
            eprintln!("Data validation failed: {}", e);
            eprintln!("Starting with empty store");
            Store::new()
        }
    };
    
    // ... rest of your program
}
```

Notice what we're doing here. All the error handling happens at this boundary. We're printing messages to stderr so the user knows what's happening. We're making policy decisions about how to handle each error type. But the library code itself just returns `Result` values and lets the caller decide what to do.

## Part 5: Determinism and Testing Persistence

Testing persistence code is challenging because you're dealing with the file system, which is stateful and potentially non-deterministic. But there are patterns that make this manageable.

### The Temp Directory Pattern

For testing, you don't want to write to fixed file paths because tests might run in parallel or leave behind artifacts that pollute other test runs. Instead, use temporary directories:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_load_empty_on_first_run() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_store_1.json");
        
        // Ensure the file doesn't exist
        let _ = fs::remove_file(&test_file);
        
        // Load should succeed and return empty store
        let result = load_from_file(&test_file);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
        
        // Clean up
        let _ = fs::remove_file(&test_file);
    }
}
```

The pattern here is to generate a unique filename in the temp directory, clean up before and after your test, and test the behavior you expect. You can use crates like `tempfile` that make this even cleaner by automatically deleting the temp directory when it goes out of scope.

### Testing Round-Trip Persistence

One of the most important tests you can write is a round-trip test: save some data, load it back, and verify you get the same data back:

```rust
#[test]
fn test_save_and_load_roundtrip() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_store_roundtrip.json");
    let _ = fs::remove_file(&test_file);
    
    // Create a store with some test data
    let mut original_store = Store::new();
    original_store.set("key1".to_string(), "value1".to_string());
    original_store.set("key2".to_string(), "value2".to_string());
    
    // Save it
    save_to_file(&test_file, &original_store).unwrap();
    
    // Load it back
    let loaded_store = load_from_file(&test_file).unwrap();
    
    // Verify it matches
    assert_eq!(original_store.len(), loaded_store.len());
    assert_eq!(original_store.get("key1"), loaded_store.get("key1"));
    assert_eq!(original_store.get("key2"), loaded_store.get("key2"));
    
    let _ = fs::remove_file(&test_file);
}
```

### A Critical Note About Determinism

Here's something that trips up many students: JSON object key ordering is not guaranteed. If you serialize a hash map to JSON, the keys might appear in different orders on different runs. This means if you do a byte-for-byte comparison of the serialized files, they might not match even though they represent the same data.

The solution is to not test based on JSON structure or key ordering. Instead, test the logical equivalence of the data. When you deserialize back into your data structures, verify that the data structures are equivalent, not that the JSON bytes are identical. If you need to test the actual file contents, you might serialize to a deterministic format or ensure your keys are sorted before serialization.

### Testing Corruption Scenarios

This is where testing gets really interesting and really important. You need to verify that your code handles corrupted data gracefully. Here's a test that deliberately creates invalid JSON:

```rust
#[test]
fn test_handles_corrupted_json() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_corrupted.json");
    
    // Write invalid JSON to the file
    fs::write(&test_file, "{ this is not valid JSON }").unwrap();
    
    // Load should fail with a serialization error, not panic
    let result = load_from_file(&test_file);
    assert!(result.is_err());
    
    match result {
        Err(PersistenceError::Serialization(_)) => {
            // This is what we expect
        }
        _ => panic!("Expected serialization error"),
    }
    
    let _ = fs::remove_file(&test_file);
}
```

This test proves that your code won't panic when it encounters garbage data. It will return a typed error that calling code can handle appropriately. You should write similar tests for partial data, missing required fields, wrong data types, and any other corruption scenario you can think of.

## Part 6: Observability and User Experience

When things go wrong with persistence, your users need to know what happened. But there's an art to providing this information in a way that's helpful without being overwhelming.

### Structured Logging at the Binary Boundary

Remember our principle about handling errors at the binary boundary? This is also where you should do your logging. Don't scatter logging statements throughout your library code. Instead, log the decisions you make in response to errors at the entry points.

```rust
fn main() {
    match load_from_file(&store_path) {
        Ok(s) => {
            eprintln!("Successfully loaded store with {} entries", s.len());
            s
        }
        Err(e) => {
            eprintln!("WARNING: Failed to load store: {:?}", e);
            eprintln!("Continuing with empty store");
            Store::new()
        }
    }
}
```

Notice we're using `eprintln!` rather than `println!`. Standard error is the appropriate stream for diagnostic and error messages. This keeps them separate from program output and makes it easier for users to redirect or filter messages.

### Error Messages Should Be Actionable

When you display an error to the user, think about what they can actually do about it. Instead of just saying "serialization failed," you might say "The store file appears to be corrupted. You may need to delete it and restart, or restore from a backup." Instead of "permission denied," you might say "Unable to write to the store file. Check that you have write permissions to the directory."

The goal is to move from "something went wrong" to "this specific thing went wrong, and here's what you might do about it."

## Part 7: Bringing It All Together

Let me show you a complete, production-quality persistence module that incorporates all the principles we've discussed:

```rust
use std::path::Path;
use std::fs;

#[derive(Debug)]
pub enum PersistenceError {
    Io(std::io::Error),
    Serialization(String),
    CorruptData(String),
}

impl From<std::io::Error> for PersistenceError {
    fn from(error: std::io::Error) -> Self {
        PersistenceError::Io(error)
    }
}

/// Load the store from disk. Returns an empty store if the file doesn't exist.
/// Returns an error for I/O failures or data corruption.
pub fn load_from_file(path: &Path) -> Result<Store, PersistenceError> {
    // Handle the file-not-found case as a valid empty state
    let contents = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Store::new());
        }
        Err(e) => return Err(PersistenceError::Io(e)),
    };

    // Empty files are also treated as empty stores
    if contents.trim().is_empty() {
        return Ok(Store::new());
    }

    // Attempt to deserialize
    let store: Store = serde_json::from_str(&contents)
        .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

    // You might add validation logic here
    // For example, checking that all keys are valid, no duplicates, etc.

    Ok(store)
}

/// Save the store to disk. Creates parent directories if needed.
pub fn save_to_file(path: &Path, store: &Store) -> Result<(), PersistenceError> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Serialize to JSON
    let contents = serde_json::to_string_pretty(store)
        .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

    // Write atomically by writing to temp file then renaming
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, contents)?;
    fs::rename(temp_path, path)?;

    Ok(())
}
```

Notice the atomic write pattern in the save function. We write to a temporary file first, then rename it to the target filename. This is a basic form of crash-safety: if the write or serialization fails partway through, we don't corrupt the existing data file. The rename operation is atomic on most operating systems, so either the old file exists or the new one does, but you never have a half-written file.

## Conclusion: The "Prove You Learned It" Checklist

Let's revisit the goals we set out to achieve and make sure you can demonstrate mastery of each one.

Can you treat a missing file as empty state while still surfacing other I/O errors? You should be able to write code that matches on `ErrorKind::NotFound` specifically and handles it differently from other I/O errors.

Can you distinguish invalid JSON from I/O failure using a typed error enum? You should be able to create an error type that wraps both categories and provides clear semantics about what went wrong.

Can you write a test that simulates corruption and verifies a non-panicking failure? You should be able to create a test that writes garbage data to a file, attempts to load it, and asserts that you get a proper error result rather than a panic.

Beyond these specific checkpoints, you should understand the deeper principles at play. Errors are not all equal; they have categories and different categories require different handling. Library code should return `Result` types and let the caller decide how to handle failures. Testing persistence requires careful thought about determinism and creating controlled scenarios. And above all, user experience matters: when things go wrong, your software should fail gracefully and provide actionable information.

This is the difference between code that works on the happy path and code that you'd trust in production. This is the difference between a student project and professional software. And now, you have the tools and understanding to build systems that handle the real world with all its messiness, uncertainty, and occasional chaos.