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

    let mut board = None;
    let mut priority = 1u8;
    let mut desc_parts = Vec::new();

    for token in args.split_whitespace() {
        if token.starts_with('@') && board.is_none() {
            board = Some(token[1..].to_string());
        } else if let Some(p) = token.strip_prefix("p:") {
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

    let mut board = None;
    let mut desc_parts = Vec::new();

    for token in args.split_whitespace() {
        if token.starts_with('@') && board.is_none() {
            board = Some(token[1..].to_string());
        } else {
            desc_parts.push(token);
        }
    }

    let description = desc_parts.join(" ");
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
    let tokens: Vec<&str> = args.split_whitespace().collect();
    if tokens.len() < 2 {
        return Err(ParseError {
            message: "Usage: /move @<id> @<board>".to_string(),
        });
    }

    let id = parse_at_id(tokens[0])?;
    let board = if tokens[1].starts_with('@') {
        tokens[1][1..].to_string()
    } else {
        tokens[1].to_string()
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
    let tokens: Vec<&str> = args.split_whitespace().collect();
    if tokens.len() < 2 {
        return Err(ParseError {
            message: "Usage: /rename-board @old @new".to_string(),
        });
    }

    let old_name = if tokens[0].starts_with('@') {
        tokens[0][1..].to_string()
    } else {
        tokens[0].to_string()
    };

    let new_name = if tokens[1].starts_with('@') {
        tokens[1][1..].to_string()
    } else {
        tokens[1].to_string()
    };

    if old_name.is_empty() || new_name.is_empty() {
        return Err(ParseError {
            message: "Board names cannot be empty".to_string(),
        });
    }

    Ok(ParsedCommand::RenameBoard { old_name, new_name })
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
