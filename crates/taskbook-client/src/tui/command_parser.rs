/// Parsed command from the command line input
#[derive(Debug, Clone)]
pub enum ParsedCommand {
    Task {
        board: Option<String>,
        description: String,
        priority: u8,
    },
    Note {
        board: Option<String>,
        description: String,
    },
    Edit {
        id: u64,
        description: String,
    },
    Move {
        id: u64,
        board: String,
    },
    Delete {
        ids: Vec<u64>,
    },
    Search {
        term: String,
    },
    Priority {
        id: u64,
        level: u8,
    },
    Check {
        ids: Vec<u64>,
    },
    Star {
        ids: Vec<u64>,
    },
    Begin {
        ids: Vec<u64>,
    },
    Clear,
    RenameBoard {
        old_name: String,
        new_name: String,
    },
    Board,
    Timeline,
    Archive,
    Journal,
    Sort,
    HideDone,
    Help,
    Quit,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Parse a command line input into a ParsedCommand
pub fn parse_command(input: &str) -> Result<ParsedCommand, ParseError> {
    let input = input.trim();
    if !input.starts_with('/') {
        return Err(ParseError {
            message: "Commands must start with /".to_string(),
        });
    }

    let parts: Vec<&str> = input[1..].splitn(2, ' ').collect();
    let cmd = parts[0].to_lowercase();
    let args = parts.get(1).copied().unwrap_or("");

    match cmd.as_str() {
        "task" => parse_task(args),
        "note" => parse_note(args),
        "edit" => parse_edit(args),
        "move" => parse_move(args),
        "delete" => parse_id_list(args).map(|ids| ParsedCommand::Delete { ids }),
        "search" => {
            let term = args.trim().to_string();
            if term.is_empty() {
                Err(ParseError {
                    message: "Usage: /search <term>".to_string(),
                })
            } else {
                Ok(ParsedCommand::Search { term })
            }
        }
        "priority" => parse_priority(args),
        "check" => parse_id_list(args).map(|ids| ParsedCommand::Check { ids }),
        "star" => parse_id_list(args).map(|ids| ParsedCommand::Star { ids }),
        "begin" => parse_id_list(args).map(|ids| ParsedCommand::Begin { ids }),
        "clear" => Ok(ParsedCommand::Clear),
        "rename-board" => parse_rename_board(args),
        "board" => Ok(ParsedCommand::Board),
        "timeline" => Ok(ParsedCommand::Timeline),
        "archive" => Ok(ParsedCommand::Archive),
        "journal" => Ok(ParsedCommand::Journal),
        "sort" => Ok(ParsedCommand::Sort),
        "hide-done" => Ok(ParsedCommand::HideDone),
        "help" => Ok(ParsedCommand::Help),
        "quit" | "q" => Ok(ParsedCommand::Quit),
        _ => Err(ParseError {
            message: format!("Unknown command: /{}", cmd),
        }),
    }
}

fn parse_task(args: &str) -> Result<ParsedCommand, ParseError> {
    let args = args.trim();
    if args.is_empty() {
        return Err(ParseError {
            message: "Usage: /task [@board] description [p:1-3]".to_string(),
        });
    }

    let (board, rest) = if args.starts_with('@') {
        match extract_at_board(args) {
            Some((name, remaining)) => (Some(name), remaining.to_string()),
            None => (None, args.to_string()),
        }
    } else {
        (None, args.to_string())
    };

    let mut priority = 1u8;
    let mut desc_parts = Vec::new();

    for token in rest.split_whitespace() {
        if let Some(p) = token.strip_prefix("p:") {
            if let Ok(v) = p.parse::<u8>() {
                if (1..=3).contains(&v) {
                    priority = v;
                }
            }
        } else {
            desc_parts.push(token);
        }
    }

    let description = desc_parts.join(" ");
    if description.is_empty() {
        return Err(ParseError {
            message: "Task description cannot be empty".to_string(),
        });
    }

    Ok(ParsedCommand::Task {
        board,
        description,
        priority,
    })
}

fn parse_note(args: &str) -> Result<ParsedCommand, ParseError> {
    let args = args.trim();
    if args.is_empty() {
        return Err(ParseError {
            message: "Usage: /note [@board] title".to_string(),
        });
    }

    let (board, rest) = if args.starts_with('@') {
        match extract_at_board(args) {
            Some((name, remaining)) => (Some(name), remaining.to_string()),
            None => (None, args.to_string()),
        }
    } else {
        (None, args.to_string())
    };

    let description = rest.trim().to_string();
    if description.is_empty() {
        return Err(ParseError {
            message: "Note title cannot be empty".to_string(),
        });
    }

    Ok(ParsedCommand::Note { board, description })
}

fn parse_edit(args: &str) -> Result<ParsedCommand, ParseError> {
    let args = args.trim();
    // Expect @<id> <description>
    let mut tokens = args.splitn(2, ' ');
    let id_token = tokens.next().unwrap_or("");
    let desc = tokens.next().unwrap_or("").trim();

    let id = parse_at_id(id_token)?;
    if desc.is_empty() {
        return Err(ParseError {
            message: "Usage: /edit @<id> <new description>".to_string(),
        });
    }

    Ok(ParsedCommand::Edit {
        id,
        description: desc.to_string(),
    })
}

fn parse_move(args: &str) -> Result<ParsedCommand, ParseError> {
    let args = args.trim();

    // Extract the ID (first token)
    let id_end = args.find(char::is_whitespace).ok_or(ParseError {
        message: "Usage: /move @<id> @<board>".to_string(),
    })?;

    let id_token = &args[..id_end];
    let rest = args[id_end..].trim();

    let id = parse_at_id(id_token)?;

    if rest.is_empty() {
        return Err(ParseError {
            message: "Usage: /move @<id> @<board>".to_string(),
        });
    }

    // Extract board name (supports @"quoted name")
    let board = if rest.starts_with('@') {
        match extract_at_board(rest) {
            Some((name, _)) => name,
            None => {
                return Err(ParseError {
                    message: "Board name cannot be empty".to_string(),
                })
            }
        }
    } else {
        // Unquoted, no @ prefix — take first word
        rest.split_whitespace()
            .next()
            .unwrap_or("")
            .to_string()
    };

    if board.is_empty() {
        return Err(ParseError {
            message: "Board name cannot be empty".to_string(),
        });
    }

    Ok(ParsedCommand::Move { id, board })
}

fn parse_priority(args: &str) -> Result<ParsedCommand, ParseError> {
    let args = args.trim();
    let tokens: Vec<&str> = args.split_whitespace().collect();
    if tokens.len() < 2 {
        return Err(ParseError {
            message: "Usage: /priority @<id> <1-3>".to_string(),
        });
    }

    let id = parse_at_id(tokens[0])?;
    let level = tokens[1].parse::<u8>().map_err(|_| ParseError {
        message: "Priority must be 1, 2, or 3".to_string(),
    })?;

    if !(1..=3).contains(&level) {
        return Err(ParseError {
            message: "Priority must be 1, 2, or 3".to_string(),
        });
    }

    Ok(ParsedCommand::Priority { id, level })
}

fn parse_rename_board(args: &str) -> Result<ParsedCommand, ParseError> {
    let args = args.trim();
    if args.is_empty() {
        return Err(ParseError {
            message: "Usage: /rename-board @\"old name\" @\"new name\"".to_string(),
        });
    }

    // Extract old board name
    let (old_name, rest) = if args.starts_with('@') {
        match extract_at_board(args) {
            Some((name, remaining)) => (name, remaining),
            None => {
                return Err(ParseError {
                    message: "Usage: /rename-board @\"old name\" @\"new name\"".to_string(),
                })
            }
        }
    } else {
        let end = args.find(char::is_whitespace).ok_or(ParseError {
            message: "Usage: /rename-board @\"old name\" @\"new name\"".to_string(),
        })?;
        (args[..end].to_string(), &args[end..])
    };

    let rest = rest.trim();

    // Extract new board name
    let new_name = if rest.starts_with('@') {
        match extract_at_board(rest) {
            Some((name, _)) => name,
            None => {
                return Err(ParseError {
                    message: "Board names cannot be empty".to_string(),
                })
            }
        }
    } else {
        rest.to_string()
    };

    if old_name.is_empty() || new_name.is_empty() {
        return Err(ParseError {
            message: "Board names cannot be empty".to_string(),
        });
    }

    Ok(ParsedCommand::RenameBoard { old_name, new_name })
}

/// Extract a board name from input starting with `@`.
///
/// Supports two forms:
/// - `@board` — single word (up to next whitespace)
/// - `@"board name"` — quoted, may contain spaces
///
/// Returns `(board_name, remaining_input)` on success.
fn extract_at_board(input: &str) -> Option<(String, &str)> {
    let input = input.trim_start();
    if !input.starts_with('@') || input.len() < 2 {
        return None;
    }

    let after_at = &input[1..]; // safe: '@' is ASCII

    if let Some(after_quote) = after_at.strip_prefix('"') {
        // Quoted: @"board name"
        if let Some(end_quote) = after_quote.find('"') {
            let board_name = &after_quote[..end_quote];
            let remaining = &after_quote[end_quote + 1..];
            if board_name.is_empty() {
                return None;
            }
            Some((board_name.to_string(), remaining))
        } else {
            // No closing quote — treat rest of string as board name
            if after_quote.is_empty() {
                return None;
            }
            Some((after_quote.to_string(), ""))
        }
    } else {
        // Unquoted: @word
        let end = after_at
            .find(char::is_whitespace)
            .unwrap_or(after_at.len());
        let board_name = &after_at[..end];
        if board_name.is_empty() {
            return None;
        }
        Some((board_name.to_string(), &after_at[end..]))
    }
}

fn parse_at_id(token: &str) -> Result<u64, ParseError> {
    let num_str = token.strip_prefix('@').unwrap_or(token);

    num_str.parse::<u64>().map_err(|_| ParseError {
        message: format!("Invalid item ID: {}", token),
    })
}

fn parse_id_list(args: &str) -> Result<Vec<u64>, ParseError> {
    let args = args.trim();
    if args.is_empty() {
        return Err(ParseError {
            message: "At least one ID is required".to_string(),
        });
    }

    let mut ids = Vec::new();
    for token in args.split_whitespace() {
        let num_str = token.strip_prefix('@').unwrap_or(token);
        let id = num_str.parse::<u64>().map_err(|_| ParseError {
            message: format!("Invalid ID: {}", token),
        })?;
        ids.push(id);
    }

    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_at_board_unquoted() {
        let (name, rest) = extract_at_board("@coding rest").unwrap();
        assert_eq!(name, "coding");
        assert_eq!(rest.trim(), "rest");
    }

    #[test]
    fn test_extract_at_board_quoted() {
        let (name, rest) = extract_at_board("@\"MiST: IT-Leder\" rest").unwrap();
        assert_eq!(name, "MiST: IT-Leder");
        assert_eq!(rest.trim(), "rest");
    }

    #[test]
    fn test_extract_at_board_quoted_no_remaining() {
        let (name, rest) = extract_at_board("@\"My Board\"").unwrap();
        assert_eq!(name, "My Board");
        assert_eq!(rest, "");
    }

    #[test]
    fn test_extract_at_board_empty_quoted() {
        assert!(extract_at_board("@\"\"").is_none());
    }

    #[test]
    fn test_extract_at_board_no_at() {
        assert!(extract_at_board("coding").is_none());
    }

    #[test]
    fn test_extract_at_board_just_at() {
        assert!(extract_at_board("@").is_none());
    }

    #[test]
    fn test_extract_at_board_unclosed_quote() {
        let (name, rest) = extract_at_board("@\"unclosed board").unwrap();
        assert_eq!(name, "unclosed board");
        assert_eq!(rest, "");
    }

    #[test]
    fn test_parse_task_quoted_board() {
        let result = parse_command("/task @\"MiST: IT-Leder\" Fix the bug").unwrap();
        match result {
            ParsedCommand::Task {
                board,
                description,
                priority,
            } => {
                assert_eq!(board.as_deref(), Some("MiST: IT-Leder"));
                assert_eq!(description, "Fix the bug");
                assert_eq!(priority, 1);
            }
            _ => panic!("Expected Task"),
        }
    }

    #[test]
    fn test_parse_task_quoted_board_with_priority() {
        let result = parse_command("/task @\"Dev Ops\" Deploy service p:3").unwrap();
        match result {
            ParsedCommand::Task {
                board,
                description,
                priority,
            } => {
                assert_eq!(board.as_deref(), Some("Dev Ops"));
                assert_eq!(description, "Deploy service");
                assert_eq!(priority, 3);
            }
            _ => panic!("Expected Task"),
        }
    }

    #[test]
    fn test_parse_task_unquoted_board() {
        let result = parse_command("/task @coding Fix bug").unwrap();
        match result {
            ParsedCommand::Task {
                board,
                description,
                ..
            } => {
                assert_eq!(board.as_deref(), Some("coding"));
                assert_eq!(description, "Fix bug");
            }
            _ => panic!("Expected Task"),
        }
    }

    #[test]
    fn test_parse_note_quoted_board() {
        let result = parse_command("/note @\"MiST: IT-Leder\" Important note").unwrap();
        match result {
            ParsedCommand::Note { board, description } => {
                assert_eq!(board.as_deref(), Some("MiST: IT-Leder"));
                assert_eq!(description, "Important note");
            }
            _ => panic!("Expected Note"),
        }
    }

    #[test]
    fn test_parse_move_quoted_board() {
        let result = parse_command("/move @1 @\"MiST: IT-Leder\"").unwrap();
        match result {
            ParsedCommand::Move { id, board } => {
                assert_eq!(id, 1);
                assert_eq!(board, "MiST: IT-Leder");
            }
            _ => panic!("Expected Move"),
        }
    }

    #[test]
    fn test_parse_rename_board_quoted() {
        let result =
            parse_command("/rename-board @\"Old Board\" @\"New Board Name\"").unwrap();
        match result {
            ParsedCommand::RenameBoard { old_name, new_name } => {
                assert_eq!(old_name, "Old Board");
                assert_eq!(new_name, "New Board Name");
            }
            _ => panic!("Expected RenameBoard"),
        }
    }

    #[test]
    fn test_parse_task_no_board() {
        let result = parse_command("/task Simple task").unwrap();
        match result {
            ParsedCommand::Task {
                board, description, ..
            } => {
                assert_eq!(board, None);
                assert_eq!(description, "Simple task");
            }
            _ => panic!("Expected Task"),
        }
    }
}
