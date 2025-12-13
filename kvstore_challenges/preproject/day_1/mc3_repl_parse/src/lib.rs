// ### Mini-Challenge 3
//
// * **Name:** REPL command parser with arity validation (no crashes)
// * **Goal:** Practice parsing commands like `SET k v`, `GET k`, `DEL k`, `LIST` safely and predictably.
// * **Setup (scratch folder)**
//
//   * `cargo new mc3_repl_parse && cd mc3_repl_parse`
//   * Create `src/lib.rs` with parsing logic + `src/main.rs` that just reads a line and prints parsed result.
// * **Requirements**
//
//   * Define:
//
//     * `enum Command { Set { key: String, val: String }, Get { key: String }, Del { key: String }, List, Exit }`
//     * `enum ParseError { Empty, UnknownCmd(String), WrongArity { cmd: String, expected: &'static str } }`
//   * Implement `fn parse_command(line: &str) -> Result<Command, ParseError>`
//   * Must handle:
//
//     * empty line
//     * extra spaces (use `split_whitespace`)
//     * wrong arg counts (typed error)
//     * unknown command
//   * Add **unit tests** for at least 6 cases:
//
//     * valid `SET a b`
//     * valid `GET a`
//     * empty input
//     * unknown cmd
//     * wrong arity for `SET`
//     * `LIST` with extra args rejected (or explicitly decide and test it)
// * **Proof**
//
//   * `cargo test` passes with 6+ parser tests
//   * `cargo run` allows typing a line and prints `Ok(...)` or `Err(...)` deterministically
// * **Guardrails**
//
//   * No `unwrap/expect` in `src/lib.rs`
//   * Deterministic display (derive `Debug` and print via `println!("{:?}", ...)`)
// * **What skill it builds for the project**
//
//   * CP3 REPL robustness (survives invalid input)
//
// ---

#[derive(Debug, PartialEq)]
enum Command {
    Get { key: String },
    Set { key: String, value: String },
    Delete { key: String },
    List,
    Exit,
}

#[derive(Debug, PartialEq)]
enum ParseError {
    Empty,
    UnknownCommand(String),
    WrongArity {
        command: String,
        expected: &'static str,
        found: usize,
    },
}

fn parse(line: &str) -> Option<(&str, Vec<&str>)> {
    let mut iter = line.split_whitespace();
    let command = iter.next()?;
    Some((command, iter.collect()))
}

fn parse_command(line: &str) -> Result<Command, ParseError> {
    let (command, args) = parse(line).ok_or(ParseError::Empty)?;

    let command = command.to_lowercase();
    let command = command.as_str();

    match command {
        "get" => {
            if args.len() != 1 {
                return Err(ParseError::WrongArity {
                    command: command.to_string(),
                    expected: "1",
                    found: args.len(),
                });
            }
            Ok(Command::Get {
                key: args[0].to_string(),
            })
        }
        "set" => {
            if args.len() != 2 {
                return Err(ParseError::WrongArity {
                    command: command.to_string(),
                    expected: "2",
                    found: args.len(),
                });
            }
            Ok(Command::Set {
                key: args[0].to_string(),
                value: args[1].to_string(),
            })
        }
        "delete" => {
            if args.len() != 1 {
                return Err(ParseError::WrongArity {
                    command: command.to_string(),
                    expected: "1",
                    found: args.len(),
                });
            }
            Ok(Command::Delete {
                key: args[0].to_string(),
            })
        }
        "list" => {
            if args.len() != 1 {
                return Err(ParseError::WrongArity {
                    command: command.to_string(),
                    expected: "1",
                    found: args.len(),
                });
            }
            Ok(Command::List)
        }
        "exit" => {
            if args.len() != 1 {
                return Err(ParseError::WrongArity {
                    command: command.to_string(),
                    expected: "1",
                    found: args.len(),
                });
            }
            Ok(Command::Exit)
        }
        _ => Err(ParseError::UnknownCommand(command.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_set_a_b() {
        assert_eq!(
            parse_command("SET a b"),
            Ok(Command::Set {
                key: "a".to_string(),
                value: "b".to_string()
            })
        );
    }

    #[test]
    fn parses_get_a_with_extra_spaces() {
        assert_eq!(
            parse_command("   GET    a   "),
            Ok(Command::Get {
                key: "a".to_string()
            })
        );
    }

    #[test]
    fn empty_input_is_error() {
        assert_eq!(parse_command("   "), Err(ParseError::Empty));
    }

    #[test]
    fn unknown_command_is_error() {
        assert_eq!(
            parse_command("FoO a"),
            Err(ParseError::UnknownCommand("foo".to_string()))
        );
    }

    #[test]
    fn wrong_arity_set_is_error() {
        assert_eq!(
            parse_command("set a"),
            Err(ParseError::WrongArity {
                command: "set".to_string(),
                expected: "2",
                found: 1
            })
        );
    }

    #[test]
    fn list_with_no_args_is_rejected() {
        assert_eq!(
            parse_command("LIST"),
            Err(ParseError::WrongArity {
                command: "list".to_string(),
                expected: "1",
                found: 0
            })
        );
    }

    #[test]
    fn parses_list_with_one_arg() {
        assert_eq!(parse_command("LIST ignored"), Ok(Command::List));
    }

    #[test]
    fn parses_delete_a() {
        assert_eq!(
            parse_command("DELETE a"),
            Ok(Command::Delete {
                key: "a".to_string()
            })
        );
    }
}
