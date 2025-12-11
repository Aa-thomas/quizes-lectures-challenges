# HashMap CRUD without panics: Building Bulletproof Key-Value Operations

Welcome to what might be the most practical lecture in this course. Today we're going to talk about something that separates hobby code from production systems: writing HashMap operations that never crash, parsing user input that never panics, and building error handling that your future self will thank you for at three in the morning when the system is down.

Let me start with a story. A few years ago, I was consulting for a startup whose entire platform went down because someone typed an extra space in a configuration command. Their REPL parser called `unwrap()` on a `None` value, the process crashed, and they lost thousands of dollars in the thirty minutes it took to restart. That panic existed because someone thought "well, users will probably type the right thing." In production systems, "probably" is not good enough.

## Why This Lecture Matters for Your KV Engine

Your key-value store project is fundamentally about three things working together correctly. First, you have HashMap semantics combined with Rust's ownership rules, which means understanding exactly when you own data versus when you're borrowing it. Second, you have a REPL that must gracefully handle every possible malformed input without crashing, because a database that crashes when you make a typo is not a database anyone will use. Third, you have the foundation for persistence, which will require clean error propagation from the lowest levels of your system up through the user interface.

Think of your KV engine this way: it's mostly just a HashMap with extremely careful attention paid to ownership rules and no-crash error flow. The complexity isn't in the data structure itself but in handling all the edge cases safely. When you add persistence later, you'll discover it's essentially "serialize and deserialize a map," but that only works if you've built clean error propagation from the start. You can't retrofit safety into a system that already panics everywhere.

## Understanding HashMap Operations and Ownership

Let me show you what happens when you interact with a `HashMap<String, String>`. When you insert a key-value pair, ownership transfers into the HashMap. The HashMap now owns those strings. This is crucial because it means the HashMap can live as long as it needs to without worrying about the original strings going away.

```rust
use std::collections::HashMap;

let mut store: HashMap<String, String> = HashMap::new();

// Ownership transfers INTO the HashMap
let key = String::from("username");
let value = String::from("alice");
store.insert(key, value);

// key and value are now moved and cannot be used anymore
// This would not compile: println!("{}", key);
```

But here's where it gets interesting. When you read from a HashMap using `get`, you don't get back owned data. You get back `Option<&V>`, which is an optional reference to the value. The HashMap is lending you a view of the data it owns. This is Rust's ownership system protecting you from accidentally removing data that other parts of your program might be looking at.

Let's think about why `get` returns `Option<&V>` rather than, say, `Option<V>` or just `&V`. The reference part makes sense because the HashMap still owns the data and you're just looking at it. The Option part exists because the key might not be in the HashMap at all. This two-layered uncertainty is exactly what makes building a clean user-facing API challenging.

```rust
// get returns Option<&String>, not Option<String>
match store.get("username") {
    Some(value_ref) => {
        // We have a reference to the String inside the HashMap
        println!("Found: {}", value_ref);
        // But we don't own it, so we can't move it out
    }
    None => {
        println!("Key not found");
    }
}
```

Now here's the pattern you'll use constantly in your KV engine: converting that `Option<&String>` into something more useful for your users. Your users don't care about Rust's borrowing rules. They care whether their key exists and what the value is. So you need to transform the HashMap's internal representation into a clean external API.

```rust
// Converting Option<&String> to Result<String, Error> for users
fn get_value(store: &HashMap<String, String>, key: &str) -> Result<String, KvError> {
    store.get(key)
        .map(|s| s.clone())  // Clone the &String to get an owned String
        .ok_or_else(|| KvError::KeyNotFound(key.to_string()))
}
```

Notice what we're doing here. We take the `Option<&String>`, use `map` to clone the reference into an owned String (because we need to return owned data to the caller), and then use `ok_or_else` to convert the None case into a proper error. This pattern appears everywhere in robust Rust code because it bridges the gap between internal implementation details and user-friendly APIs.

## Typed Errors Versus Stringly-Typed Errors

Let me show you two ways to handle errors, and then I'll explain why one of them will cause you pain in production. First, the naive approach that many beginners take:

```rust
// The stringly-typed approach (AVOID THIS)
fn get_user(store: &HashMap<String, String>, key: &str) -> Result<String, String> {
    store.get(key)
        .map(|s| s.clone())
        .ok_or_else(|| format!("Key '{}' not found", key))
}

fn parse_command(input: &str) -> Result<Command, String> {
    if input.is_empty() {
        return Err("Empty input".to_string());
    }
    // ... more parsing
    Err("Invalid command".to_string())
}
```

This looks reasonable at first glance. You're returning error messages as strings, which seems natural since errors are meant to be read by humans. But think about what happens when you need to handle these errors differently based on what went wrong. How do you distinguish between "key not found" and "invalid command"? You end up doing string comparisons, which are fragile and break when you change your error messages.

Now consider the typed approach using an enum:

```rust
// The typed approach (EMBRACE THIS)
#[derive(Debug)]
enum KvError {
    KeyNotFound(String),
    EmptyInput,
    InvalidCommand(String),
    WrongArgumentCount { expected: usize, got: usize },
}

impl std::fmt::Display for KvError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            KvError::KeyNotFound(key) => write!(f, "Key '{}' not found", key),
            KvError::EmptyInput => write!(f, "Empty input provided"),
            KvError::InvalidCommand(cmd) => write!(f, "Invalid command: {}", cmd),
            KvError::WrongArgumentCount { expected, got } => {
                write!(f, "Expected {} arguments, got {}", expected, got)
            }
        }
    }
}

impl std::error::Error for KvError {}
```

With typed errors, your code can now make decisions based on the error type. Maybe "key not found" should log at debug level while "wrong argument count" should log at warning level. Maybe "empty input" should silently prompt for input again while "invalid command" should show a help message. These distinctions are impossible with string errors but trivial with typed errors.

Here's the key insight: errors are not just messages for humans. They're data that flows through your program, and like all data, they benefit from having a proper type. In production, you'll want to log errors to different places, retry certain operations, or handle errors differently based on their cause. Typed errors make all of this straightforward.

## The Library Code Mindset: No Panics Allowed

Let me introduce you to a concept I call "library-style code." When you write a library, you cannot make assumptions about how it will be used. You cannot assume the user will provide valid input. You cannot assume the network will be reliable. You cannot assume anything. Every function must either succeed or return a descriptive error.

In application code, you might write this:

```rust
// Application-style code (sometimes acceptable at the top level)
let value = store.get(&key).expect("Key must exist");
```

That `expect` is a panic waiting to happen. It's saying "if this key doesn't exist, crash the entire program." In a small script, maybe that's okay. In a database that someone else is relying on, it's completely unacceptable.

Library-style code never uses `unwrap()` or `expect()` except in tests or in tiny demo code. Instead, it uses the question mark operator to propagate errors up the call stack:

```rust
// Library-style code (always use this in production)
fn process_user_command(store: &mut HashMap<String, String>, cmd: &str) -> Result<String, KvError> {
    let parsed = parse_command(cmd)?;  // Propagates ParseError up
    let result = execute_command(store, parsed)?;  // Propagates ExecuteError up
    Ok(result)
}
```

That question mark operator is doing something subtle and powerful. It's saying "if this operation fails, convert the error into my return type and return early." This creates a chain of error handling where each function is responsible for its own invariants but errors flow naturally up to where they can be handled meaningfully.

Think about what happens in a real REPL. The user types something, your parser tries to parse it, your executor tries to execute it, and somewhere at the top level you display either the result or an error message. With library-style code and the question mark operator, errors flow naturally from the bottom of the stack (where they occur) to the top (where they're displayed), and every layer in between remains clean and focused.

```rust
// The top-level REPL loop is the ONLY place that handles errors for display
loop {
    let input = read_line()?;
    
    match process_user_command(&mut store, &input) {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("Error: {}", e),  // Log to stderr
    }
}
```

Notice how the top level distinguishes between normal output (which goes to stdout with `println`) and errors (which go to stderr with `eprintln`). This is crucial for production systems where you want to be able to redirect output and errors to different places.

## Determinism and Testing: Why Sorted Output Matters

Here's a subtle problem that will break your tests and confuse your users: HashMaps do not maintain insertion order. The order you get when you iterate over a HashMap is effectively random. It might be consistent in one run, but it's not guaranteed across runs or across different machines.

```rust
let mut store = HashMap::new();
store.insert("zebra".to_string(), "stripes".to_string());
store.insert("aardvark".to_string(), "long tongue".to_string());
store.insert("moose".to_string(), "antlers".to_string());

// This order is UNDEFINED
for (key, value) in &store {
    println!("{}: {}", key, value);
}
```

Why does this matter? Because when you write tests, you want them to be deterministic. You want the same input to produce the same output every single time. And when users are debugging issues, they want to be able to compare outputs and see exactly what changed.

The solution is simple but requires discipline: always sort your keys before displaying them.

```rust
fn list_all(store: &HashMap<String, String>) -> String {
    // Collect keys into a Vec so we can sort them
    let mut keys: Vec<_> = store.keys().collect();
    keys.sort();  // Sort the keys alphabetically
    
    // Now iterate in sorted order
    let mut output = String::new();
    for key in keys {
        let value = &store[key];  // Safe because we know the key exists
        output.push_str(&format!("{}: {}\n", key, value));
    }
    output
}
```

This pattern of "collect, sort, then iterate" appears constantly in production code. It's how you make HashMaps testable. It's how you make output comparable. It's how you make debugging possible. Without it, your tests will randomly fail and you'll spend hours trying to figure out why the same data produces different output.

## REPL Parsing Foundations: Handling Human Input Safely

Now let's talk about parsing user input, which is where most panics in interactive programs originate. Users are unpredictable. They'll type commands with extra spaces, missing arguments, trailing whitespace, empty lines, and every other variation you haven't thought of. Your parser must handle all of it gracefully.

Here's the foundation of safe command parsing:

```rust
fn parse(line: &str) -> Option<(&str, Vec<&str>)> {
    let mut it = line.split_whitespace();
    let cmd = it.next()?;  // Returns None if the line is empty
    Some((cmd, it.collect()))  // Collect remaining parts as arguments
}
```

Let me walk through what this tiny function does. First, it splits the input line on whitespace, which handles multiple spaces, tabs, and leading/trailing whitespace automatically. Then it tries to get the first word as the command. If the line is empty or contains only whitespace, `next()` returns `None`, which the question mark operator propagates immediately. If there is a command, we collect the remaining words into a vector and return both.

This function is deceptively simple, but it handles several edge cases: empty input returns `None` rather than panicking. Extra whitespace is ignored naturally by `split_whitespace`. Commands with no arguments work fine because `collect()` on an empty iterator gives you an empty vector.

Now let's build on this foundation to create a proper command enum:

```rust
#[derive(Debug, PartialEq)]
enum Command {
    Get { key: String },
    Set { key: String, value: String },
    Delete { key: String },
    List,
    Exit,
}

fn parse_command(line: &str) -> Result<Command, KvError> {
    // First, check for empty input
    let (cmd, args) = parse(line).ok_or(KvError::EmptyInput)?;
    
    // Now match on the command and validate argument counts
    match cmd.to_lowercase().as_str() {
        "get" => {
            if args.len() != 1 {
                return Err(KvError::WrongArgumentCount { expected: 1, got: args.len() });
            }
            Ok(Command::Get { key: args[0].to_string() })
        }
        "set" => {
            if args.len() != 2 {
                return Err(KvError::WrongArgumentCount { expected: 2, got: args.len() });
            }
            Ok(Command::Set { 
                key: args[0].to_string(),
                value: args[1].to_string() 
            })
        }
        "delete" => {
            if args.len() != 1 {
                return Err(KvError::WrongArgumentCount { expected: 1, got: args.len() });
            }
            Ok(Command::Delete { key: args[0].to_string() })
        }
        "list" => {
            if !args.is_empty() {
                return Err(KvError::WrongArgumentCount { expected: 0, got: args.len() });
            }
            Ok(Command::List)
        }
        "exit" => {
            if !args.is_empty() {
                return Err(KvError::WrongArgumentCount { expected: 0, got: args.len() });
            }
            Ok(Command::Exit)
        }
        _ => Err(KvError::InvalidCommand(cmd.to_string())),
    }
}
```

Notice the pattern here: every branch validates its argument count before constructing the command. This is arity validation, which is just a fancy way of saying "checking that you got the right number of arguments." It's tedious but essential. Without it, you'd panic when trying to access `args[1]` on a command that only provided one argument.

The function also normalizes the command to lowercase, which means users can type "GET", "get", or "Get" and they all work. This is the kind of user-friendly behavior that makes the difference between a tool people tolerate and a tool people enjoy using.

## Observability: Logging for Future Debugging

Here's something that experienced engineers know but that often gets left out of university courses: when your program misbehaves in production, you'll wish you had logged more. Observability means making your program's behavior visible so you can understand what it's doing and why.

Even in drill exercises, get into the habit of logging important operations to stderr:

```rust
fn execute_command(store: &mut HashMap<String, String>, cmd: Command) -> Result<String, KvError> {
    match cmd {
        Command::Set { key, value } => {
            eprintln!("[DEBUG] Setting key='{}' to value='{}'", key, value);
            store.insert(key.clone(), value);
            Ok(format!("Set key '{}'", key))
        }
        Command::Get { key } => {
            eprintln!("[DEBUG] Getting key='{}'", key);
            store.get(&key)
                .map(|v| v.clone())
                .ok_or_else(|| KvError::KeyNotFound(key))
        }
        Command::Delete { key } => {
            eprintln!("[DEBUG] Deleting key='{}'", key);
            store.remove(&key)
                .map(|_| format!("Deleted key '{}'", key))
                .ok_or_else(|| KvError::KeyNotFound(key))
        }
        Command::List => {
            eprintln!("[DEBUG] Listing all keys");
            Ok(list_all(store))
        }
        Command::Exit => {
            eprintln!("[DEBUG] Exit command received");
            Ok("Goodbye".to_string())
        }
    }
}
```

Why stderr instead of stdout? Because stdout is for program output that users or other programs will consume. Stderr is for diagnostics. This separation means you can redirect output to a file while still seeing debug information, or you can suppress debug information while preserving output. In production, you'd replace these `eprintln` calls with a proper logging framework, but the principle remains the same.

## Putting It All Together: A Complete Example

Let me show you how all these pieces fit together in a minimal but complete KV store implementation:

```rust
use std::collections::HashMap;

#[derive(Debug)]
enum KvError {
    KeyNotFound(String),
    EmptyInput,
    InvalidCommand(String),
    WrongArgumentCount { expected: usize, got: usize },
}

impl std::fmt::Display for KvError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            KvError::KeyNotFound(k) => write!(f, "Key '{}' not found", k),
            KvError::EmptyInput => write!(f, "Empty input"),
            KvError::InvalidCommand(c) => write!(f, "Invalid command: {}", c),
            KvError::WrongArgumentCount { expected, got } => {
                write!(f, "Expected {} args, got {}", expected, got)
            }
        }
    }
}

// Parse splits input into command and arguments
fn parse(line: &str) -> Option<(&str, Vec<&str>)> {
    let mut it = line.split_whitespace();
    let cmd = it.next()?;
    Some((cmd, it.collect()))
}

// Convert Option<&String> to Result<String, KvError> safely
fn get_value(store: &HashMap<String, String>, key: &str) -> Result<String, KvError> {
    store.get(key)
        .map(|s| s.clone())
        .ok_or_else(|| KvError::KeyNotFound(key.to_string()))
}

// List all keys in sorted order for deterministic output
fn list_all(store: &HashMap<String, String>) -> String {
    let mut keys: Vec<_> = store.keys().collect();
    keys.sort();
    keys.iter()
        .map(|k| format!("{}: {}", k, store[k.as_str()]))
        .collect::<Vec<_>>()
        .join("\n")
}
```

## Your "Prove You Learned It" Checklist

To verify that you've truly internalized these concepts, you should be able to complete these tasks without looking back at the notes:

You should be able to write a function that takes `Option<&String>` from a HashMap get operation and converts it into `Result<String, MyError>` without cloning unnecessarily. The key insight here is understanding when cloning is necessary (when you need owned data to return) versus when you can avoid it (when you're just checking existence).

You should be able to write a complete command parser that handles empty input, validates argument counts for each command, and returns typed errors for every failure case. Your parser should have tests that verify it correctly rejects empty strings, wrong argument counts, and unknown commands.

You should be able to implement a list function that produces deterministic output regardless of HashMap insertion order. This means collecting keys, sorting them, and then formatting the output consistently.

Each of these skills represents a fundamental pattern you'll use throughout the project. The HashMap to Result conversion is how you build user-facing APIs over low-level data structures. The safe parsing is how you build reliable interactive systems. The deterministic output is how you make your code testable and debuggable.

## Conclusion: Building Systems That Don't Crash

Everything we've covered today comes down to one principle: production systems must handle every possible input and error condition gracefully. Users will provide malformed input. Keys won't exist. Arguments will be missing or excessive. Your code must anticipate all of this and respond with clear, helpful errors rather than panicking.

The patterns we've discussed, using typed errors instead of strings, propagating errors with the question mark operator, validating input before processing it, are not just academic exercises. They're the difference between code that works in demos and code that works in production. They're the difference between a system that crashes mysteriously and a system that explains exactly what went wrong and how to fix it.

As you build your KV engine, remember that every `unwrap()` is a potential production incident waiting to happen. Every unvalidated input is a panic that hasn't occurred yet. Every unsorted output is a test that will randomly fail. Build with library-style discipline from the start, and you'll thank yourself later.
