use super::app::{App, Suggestion, SuggestionKind};
use taskbook_common::board;

/// Static list of all slash commands with descriptions
const COMMANDS: &[(&str, &str)] = &[
    ("task", "Create a new task"),
    ("note", "Create a new note"),
    ("edit", "Edit item description"),
    ("move", "Move item to board"),
    ("delete", "Delete items"),
    ("search", "Search/filter items"),
    ("priority", "Set task priority"),
    ("check", "Toggle task check"),
    ("star", "Toggle star"),
    ("begin", "Toggle in-progress"),
    ("clear", "Clear completed tasks"),
    ("rename-board", "Rename a board"),
    ("board", "Switch to board view"),
    ("timeline", "Switch to timeline view"),
    ("archive", "Switch to archive view"),
    ("journal", "Switch to journal view"),
    ("sort", "Cycle sort method"),
    ("hide-done", "Toggle hide completed"),
    ("help", "Show help"),
];

/// Commands that accept item ID references (@<id>)
const ITEM_COMMANDS: &[&str] = &["check", "star", "begin", "delete", "edit", "move", "priority"];

const MAX_SUGGESTIONS: usize = 8;

/// Update suggestions based on current command line input
pub fn update_suggestions(app: &mut App) {
    app.command_line.suggestions.clear();
    app.command_line.selected_suggestion = None;

    let input = &app.command_line.input;
    if input.is_empty() || !input.starts_with('/') {
        return;
    }

    // Find the token being typed by scanning backward from cursor
    let chars: Vec<char> = input.chars().collect();
    let cursor = app.command_line.cursor.min(chars.len());

    // Get the text up to the cursor for analysis
    let text_to_cursor: String = chars[..cursor].iter().collect();

    // Check what context we're in
    if !text_to_cursor.contains(' ') {
        // Still typing the command name (e.g., "/ta")
        let partial = &text_to_cursor[1..]; // skip the '/'
        suggest_commands(app, partial);
    } else {
        // We're past the command name — determine context
        let space_pos = text_to_cursor.find(' ').unwrap();
        let command = &text_to_cursor[1..space_pos]; // skip '/'

        // Find the last token start (use char index, not byte index)
        let last_space = chars[..cursor].iter().rposition(|c| *c == ' ').unwrap();
        let last_token: String = chars[last_space + 1..cursor].iter().collect();

        if let Some(after_at) = last_token.strip_prefix('@') {
            // If it's @<digits>, user is typing an ID directly — no suggestions
            if !after_at.is_empty() && after_at.chars().all(|c| c.is_ascii_digit()) {
                return;
            }
            // Otherwise it's a board reference
            suggest_boards(app, after_at);
        } else if ITEM_COMMANDS.contains(&command) {
            // Check if we should suggest items for this argument position
            if should_suggest_items(command, &text_to_cursor, last_space) {
                suggest_items(app, &last_token);
            }
        }
    }
}

/// Determine whether the current argument position should get item suggestions
fn should_suggest_items(command: &str, text_to_cursor: &str, _last_space: usize) -> bool {
    let after_command = text_to_cursor.split_once(' ').map(|(_, rest)| rest).unwrap_or("");
    let args: Vec<&str> = after_command.split_whitespace().collect();

    match command {
        // /edit @<id> <description> — only suggest for the first argument
        "edit" => args.len() <= 1,
        // /move @<id> @<board> — only suggest for the first argument
        "move" => args.len() <= 1,
        // /priority @<id> <1-3> — only suggest for the first argument
        "priority" => args.len() <= 1,
        // Multi-ID commands: check, star, begin, delete — always suggest
        _ => true,
    }
}

fn suggest_commands(app: &mut App, partial: &str) {
    let partial_lower = partial.to_lowercase();
    for (name, desc) in COMMANDS {
        if name.starts_with(&partial_lower) {
            app.command_line.suggestions.push(Suggestion {
                display: format!("/{}", name),
                completion: format!("/{} ", name),
                description: Some(desc.to_string()),
                kind: SuggestionKind::Command,
            });
            if app.command_line.suggestions.len() >= MAX_SUGGESTIONS {
                break;
            }
        }
    }
}

fn suggest_boards(app: &mut App, partial: &str) {
    let partial_lower = partial.to_lowercase();
    let board_names: Vec<String> = app.boards.clone();

    for b in &board_names {
        let display = board::display_name(b);
        if display.to_lowercase().starts_with(&partial_lower)
            || b.to_lowercase().starts_with(&partial_lower)
        {
            // Build the completion: replace the @partial with @board
            let input_chars: Vec<char> = app.command_line.input.chars().collect();
            let cursor = app.command_line.cursor.min(input_chars.len());

            // Find the @ position (use char index, not byte index)
            if let Some(at_pos) = input_chars[..cursor].iter().rposition(|c| *c == '@') {
                let before_at: String = input_chars[..at_pos].iter().collect();
                let after_cursor: String = input_chars[cursor..].iter().collect();
                let completion = format!("{}@{} {}", before_at, b, after_cursor);

                app.command_line.suggestions.push(Suggestion {
                    display: format!("@{}", display),
                    completion,
                    description: None,
                    kind: SuggestionKind::Board,
                });
            }

            if app.command_line.suggestions.len() >= MAX_SUGGESTIONS {
                break;
            }
        }
    }
}

fn suggest_items(app: &mut App, partial: &str) {
    if partial.is_empty() {
        return;
    }

    let partial_lower = partial.to_lowercase();
    let input_chars: Vec<char> = app.command_line.input.chars().collect();
    let cursor = app.command_line.cursor.min(input_chars.len());
    let last_space = input_chars[..cursor]
        .iter()
        .rposition(|c| *c == ' ')
        .unwrap_or(0);

    // Collect and sort by ID for stable ordering
    let mut matches: Vec<(u64, &str, bool, bool, bool)> = app
        .items
        .values()
        .filter(|item| item.description().to_lowercase().contains(&partial_lower))
        .map(|item| {
            let (is_complete, in_progress) = match item {
                taskbook_common::StorageItem::Task(t) => (t.is_complete, t.in_progress),
                taskbook_common::StorageItem::Note(_) => (false, false),
            };
            (
                item.id(),
                item.description(),
                item.is_task(),
                is_complete,
                in_progress,
            )
        })
        .collect();
    matches.sort_by_key(|(id, _, _, _, _)| *id);

    for (id, desc, is_task, is_complete, in_progress) in matches {
        // Build status description
        let type_label = if is_task { "task" } else { "note" };
        let status = if is_complete {
            format!("{} · done", type_label)
        } else if in_progress {
            format!("{} · in-progress", type_label)
        } else {
            type_label.to_string()
        };

        // Truncate description for display
        let display: String = if desc.len() > 35 {
            format!("{}…", desc.chars().take(34).collect::<String>())
        } else {
            desc.to_string()
        };

        // Build completion: replace the partial token with @<id>
        let before_token: String = input_chars[..last_space + 1].iter().collect();
        let after_cursor: String = input_chars[cursor..].iter().collect();
        let completion = format!("{}@{} {}", before_token, id, after_cursor);

        app.command_line.suggestions.push(Suggestion {
            display,
            completion,
            description: Some(status),
            kind: SuggestionKind::Item,
        });

        if app.command_line.suggestions.len() >= MAX_SUGGESTIONS {
            break;
        }
    }
}
