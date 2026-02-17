use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::editor;
use crate::error::Result;
use taskbook_common::board;

use super::app::{App, PendingAction, PopupState, StatusKind, ViewMode};
use super::autocomplete;
use super::command_parser::{self, ParsedCommand};
use super::input_handler::{handle_text_input, InputResult};

/// Handle a key event
pub fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<()> {
    // 1. Help popup → any key dismisses
    if app.popup.is_some() {
        app.popup = None;
        return Ok(());
    }

    // 2. Pending confirm → Enter/Esc only
    if app.command_line.pending_confirm.is_some() {
        return handle_confirm_key(app, key);
    }

    // 3. Command line focused → handle command line input
    if app.command_line.focused {
        return handle_command_line_key(app, key);
    }

    // 4. Normal mode shortcuts
    handle_shortcut_key(app, key)
}

/// Handle keys when a confirmation is pending
fn handle_confirm_key(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Enter => {
            if let Some(action) = app.command_line.pending_confirm.take() {
                match action {
                    PendingAction::Delete { ids } => {
                        delete_items(app, &ids)?;
                    }
                    PendingAction::Clear => {
                        clear_completed(app)?;
                    }
                }
            }
            app.deactivate_command_line();
        }
        KeyCode::Esc => {
            app.command_line.pending_confirm = None;
            app.deactivate_command_line();
        }
        _ => {}
    }
    Ok(())
}

/// Handle keys when the command line is focused
fn handle_command_line_key(app: &mut App, key: KeyEvent) -> Result<()> {
    // Tab accepts the selected suggestion
    if key.code == KeyCode::Tab {
        accept_suggestion(app);
        return Ok(());
    }

    // Up/Down navigate suggestions (only if suggestions exist)
    if !app.command_line.suggestions.is_empty() {
        match key.code {
            KeyCode::Up => {
                let count = app.command_line.suggestions.len();
                app.command_line.selected_suggestion = Some(match app.command_line.selected_suggestion
                {
                    None => count - 1,
                    Some(0) => count - 1,
                    Some(i) => i - 1,
                });
                return Ok(());
            }
            KeyCode::Down => {
                let count = app.command_line.suggestions.len();
                app.command_line.selected_suggestion =
                    Some(match app.command_line.selected_suggestion {
                        None => 0,
                        Some(i) if i + 1 >= count => 0,
                        Some(i) => i + 1,
                    });
                return Ok(());
            }
            _ => {}
        }
    }

    // Use the existing text input handler for editing
    let input = app.command_line.input.clone();
    let cursor = app.command_line.cursor;

    match handle_text_input(key, &input, cursor) {
        InputResult::Cancel => {
            app.deactivate_command_line();
        }
        InputResult::Submit => {
            let input = app.command_line.input.clone();
            app.deactivate_command_line();
            if !input.trim().is_empty() {
                execute_input(app, &input)?;
            }
        }
        InputResult::Changed {
            input: new_input,
            cursor: new_cursor,
        } => {
            app.command_line.input = new_input;
            app.command_line.cursor = new_cursor;
            autocomplete::update_suggestions(app);
        }
        InputResult::Ignored => {}
    }

    Ok(())
}

/// Accept the currently selected suggestion
fn accept_suggestion(app: &mut App) {
    let selected = app.command_line.selected_suggestion.unwrap_or(0);
    if let Some(suggestion) = app.command_line.suggestions.get(selected).cloned() {
        app.command_line.input = suggestion.completion.clone();
        app.command_line.cursor = suggestion.completion.chars().count();
        app.command_line.suggestions.clear();
        app.command_line.selected_suggestion = None;
        // Re-trigger suggestions for the new input
        autocomplete::update_suggestions(app);
    }
}

/// Parse and execute the command line input
fn execute_input(app: &mut App, input: &str) -> Result<()> {
    match command_parser::parse_command(input) {
        Ok(cmd) => execute_command(app, cmd),
        Err(e) => {
            app.set_status(e.message, StatusKind::Error);
            Ok(())
        }
    }
}

/// Execute a parsed command
fn execute_command(app: &mut App, cmd: ParsedCommand) -> Result<()> {
    match cmd {
        ParsedCommand::Task {
            board,
            description,
            priority,
        } => {
            let board_name = board
                .map(|b| board::normalize_board_name(&b))
                .or_else(|| app.filter.board_filter.clone())
                .unwrap_or_else(|| "my board".to_string());
            app.taskbook
                .create_task_direct(vec![board_name.clone()], description, priority)?;
            app.refresh_items()?;
            let display = board::display_name(&board_name);
            app.set_status(format!("Task created in {}", display), StatusKind::Success);
        }
        ParsedCommand::Note { board, description } => {
            let board_name = board
                .map(|b| board::normalize_board_name(&b))
                .or_else(|| app.filter.board_filter.clone())
                .unwrap_or_else(|| "my board".to_string());
            app.taskbook
                .create_note_direct(vec![board_name.clone()], description)?;
            app.refresh_items()?;
            let display = board::display_name(&board_name);
            app.set_status(format!("Note created in {}", display), StatusKind::Success);

            // Open editor for the newly created note
            // Find the highest ID note (just created)
            if let Some(max_id) = app.items.values().map(|i| i.id()).max() {
                if let Some(item) = app.items.get(&max_id.to_string()) {
                    if !item.is_task() {
                        edit_note_external(app, max_id)?;
                    }
                }
            }
        }
        ParsedCommand::Edit { id, description } => {
            edit_description(app, id, &description)?;
        }
        ParsedCommand::Move { id, board } => {
            move_to_board(app, id, &board)?;
        }
        ParsedCommand::Delete { ids } => {
            app.command_line.pending_confirm = Some(PendingAction::Delete { ids });
        }
        ParsedCommand::Search { term } => {
            app.filter.search_term = Some(term.clone());
            app.update_display_order();
            app.selected_index = 0;
            let count = app.display_order.len();
            app.set_status(
                format!("Search: \"{}\" ({} matches)", term, count),
                StatusKind::Info,
            );
        }
        ParsedCommand::Priority { id, level } => {
            set_priority(app, id, level)?;
        }
        ParsedCommand::Check { ids } => {
            for id in &ids {
                toggle_check(app, *id)?;
            }
        }
        ParsedCommand::Star { ids } => {
            for id in &ids {
                toggle_star(app, *id)?;
            }
        }
        ParsedCommand::Begin { ids } => {
            for id in &ids {
                toggle_begin(app, *id)?;
            }
        }
        ParsedCommand::Clear => {
            app.command_line.pending_confirm = Some(PendingAction::Clear);
        }
        ParsedCommand::RenameBoard { old_name, new_name } => {
            rename_board(app, &old_name, &new_name)?;
        }
        ParsedCommand::Board => {
            app.clear_board_filter();
            app.set_view(ViewMode::Board)?;
        }
        ParsedCommand::Timeline => {
            app.clear_board_filter();
            app.set_view(ViewMode::Timeline)?;
        }
        ParsedCommand::Archive => {
            app.clear_board_filter();
            app.set_view(ViewMode::Archive)?;
        }
        ParsedCommand::Journal => {
            app.clear_board_filter();
            app.set_view(ViewMode::Journal)?;
        }
        ParsedCommand::Sort => {
            app.cycle_sort_method();
            app.set_status(
                format!("Sort: {}", app.sort_method.display_name()),
                StatusKind::Info,
            );
        }
        ParsedCommand::HideDone => {
            app.toggle_hide_completed();
            let msg = if app.filter.hide_completed {
                "Hiding completed tasks"
            } else {
                "Showing completed tasks"
            };
            app.set_status(msg.to_string(), StatusKind::Info);
        }
        ParsedCommand::Help => {
            app.popup = Some(PopupState::Help);
        }
    }
    Ok(())
}

/// Handle shortcut keys in normal (unfocused) mode
fn handle_shortcut_key(app: &mut App, key: KeyEvent) -> Result<()> {
    // Ctrl shortcuts should not trigger single-char shortcuts
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return Ok(());
    }

    match key.code {
        // Quit
        KeyCode::Char('q') => app.quit(),
        KeyCode::Esc => {
            if app.filter.search_term.is_some() {
                app.filter.search_term = None;
                app.update_display_order();
                app.selected_index = 0;
                app.set_status("Search cleared".to_string(), StatusKind::Info);
            } else if app.filter.board_filter.is_some() {
                app.clear_board_filter();
                app.set_status("Filter cleared".to_string(), StatusKind::Info);
            } else {
                app.quit();
            }
        }

        // Navigation
        KeyCode::Char('j') | KeyCode::Down => app.select_next(),
        KeyCode::Char('k') | KeyCode::Up => app.select_previous(),
        KeyCode::Char('g') => app.select_first(),
        KeyCode::Char('G') => app.select_last(),

        // Enter to open note in editor or filter by board
        KeyCode::Enter => {
            if let Some(item) = app.selected_item() {
                if !item.is_task() {
                    edit_note_external(app, item.id())?;
                } else if app.view == ViewMode::Board && app.filter.board_filter.is_none() {
                    if let Some(board_name) = app.get_board_for_selected() {
                        let display = board::display_name(&board_name);
                        app.set_board_filter(Some(board_name));
                        app.set_status(format!("Filtering by {}", display), StatusKind::Info);
                    }
                }
            } else if app.view == ViewMode::Board && app.filter.board_filter.is_none() {
                if let Some(board_name) = app.get_board_for_selected() {
                    let display = board::display_name(&board_name);
                    app.set_board_filter(Some(board_name));
                    app.set_status(format!("Filtering by {}", display), StatusKind::Info);
                }
            }
        }

        // View switching
        KeyCode::Char('1') => {
            app.clear_board_filter();
            app.set_view(ViewMode::Board)?;
        }
        KeyCode::Char('2') => {
            app.clear_board_filter();
            app.set_view(ViewMode::Timeline)?;
        }
        KeyCode::Char('3') => {
            app.clear_board_filter();
            app.set_view(ViewMode::Archive)?;
        }
        KeyCode::Char('4') => {
            app.clear_board_filter();
            app.set_view(ViewMode::Journal)?;
        }

        // Help
        KeyCode::Char('?') => {
            app.popup = Some(PopupState::Help);
        }

        // Slash or Tab activates command line
        KeyCode::Char('/') | KeyCode::Tab => {
            app.activate_command_line("/");
            autocomplete::update_suggestions(app);
        }

        // Pre-fill shortcuts — activate command line with partial command
        KeyCode::Char('t') if app.view != ViewMode::Archive => {
            if let Some(ref board) = app.filter.board_filter.clone() {
                app.activate_command_line(&format!("/task @{} ", board));
            } else {
                app.activate_command_line("/task @");
                autocomplete::update_suggestions(app);
            }
        }
        KeyCode::Char('n') if app.view != ViewMode::Archive => {
            if let Some(ref board) = app.filter.board_filter.clone() {
                app.activate_command_line(&format!("/note @{} ", board));
            } else {
                app.activate_command_line("/note @");
                autocomplete::update_suggestions(app);
            }
        }
        KeyCode::Char('e') if app.view != ViewMode::Archive => {
            if let Some(item) = app.selected_item() {
                let id = item.id();
                let desc = item.description().to_string();
                app.activate_command_line(&format!("/edit @{} {}", id, desc));
            }
        }
        KeyCode::Char('m') if app.view != ViewMode::Archive => {
            if let Some(id) = app.selected_id() {
                app.activate_command_line(&format!("/move @{} @", id));
                autocomplete::update_suggestions(app);
            }
        }
        KeyCode::Char('p') if app.view != ViewMode::Archive => {
            if let Some(item) = app.selected_item() {
                if item.is_task() {
                    app.activate_command_line(&format!("/priority @{} ", item.id()));
                }
            }
        }
        KeyCode::Char('d') if app.view != ViewMode::Archive => {
            if let Some(id) = app.selected_id() {
                app.command_line.pending_confirm = Some(PendingAction::Delete { ids: vec![id] });
            }
        }
        KeyCode::Char('C') if app.view != ViewMode::Archive => {
            app.command_line.pending_confirm = Some(PendingAction::Clear);
        }

        // Direct action shortcuts (no command line needed)
        KeyCode::Char('c') if app.view != ViewMode::Archive => {
            if let Some(id) = app.selected_id() {
                toggle_check(app, id)?;
            }
        }
        KeyCode::Char('b') if app.view != ViewMode::Archive => {
            if let Some(id) = app.selected_id() {
                toggle_begin(app, id)?;
            }
        }
        KeyCode::Char('s') if app.view != ViewMode::Archive => {
            if let Some(id) = app.selected_id() {
                toggle_star(app, id)?;
            }
        }
        KeyCode::Char('r') if app.view == ViewMode::Archive => {
            if let Some(id) = app.selected_id() {
                restore_item(app, id)?;
            }
        }
        KeyCode::Char('y') => {
            if let Some(id) = app.selected_id() {
                copy_to_clipboard(app, id)?;
            }
        }

        // Cycle sort method
        KeyCode::Char('S') if app.view == ViewMode::Board => {
            app.cycle_sort_method();
            app.set_status(
                format!("Sort: {}", app.sort_method.display_name()),
                StatusKind::Info,
            );
        }
        // Toggle hide completed
        KeyCode::Char('h') if app.view != ViewMode::Archive => {
            app.toggle_hide_completed();
            let msg = if app.filter.hide_completed {
                "Hiding completed tasks"
            } else {
                "Showing completed tasks"
            };
            app.set_status(msg.to_string(), StatusKind::Info);
        }

        _ => {}
    }

    Ok(())
}

// Action implementations

fn toggle_check(app: &mut App, id: u64) -> Result<()> {
    if let Some(item) = app.items.get(&id.to_string()) {
        if item.is_task() {
            app.taskbook.check_tasks_silent(&[id])?;
            app.refresh_items()?;
            app.set_status(format!("Toggled task {}", id), StatusKind::Success);
        }
    }
    Ok(())
}

fn toggle_begin(app: &mut App, id: u64) -> Result<()> {
    if let Some(item) = app.items.get(&id.to_string()) {
        if item.is_task() {
            app.taskbook.begin_tasks_silent(&[id])?;
            app.refresh_items()?;
            app.set_status(
                format!("Toggled in-progress for task {}", id),
                StatusKind::Success,
            );
        }
    }
    Ok(())
}

fn toggle_star(app: &mut App, id: u64) -> Result<()> {
    app.taskbook.star_items_silent(&[id])?;
    app.refresh_items()?;
    app.set_status(format!("Toggled star for item {}", id), StatusKind::Success);
    Ok(())
}

fn edit_description(app: &mut App, id: u64, new_desc: &str) -> Result<()> {
    app.taskbook.edit_description_silent(id, new_desc)?;
    app.refresh_items()?;
    app.set_status(format!("Updated item {}", id), StatusKind::Success);
    Ok(())
}

fn move_to_board(app: &mut App, id: u64, board: &str) -> Result<()> {
    let board_name = board::normalize_board_name(board);
    app.taskbook
        .move_boards_silent(id, vec![board_name.clone()])?;
    app.refresh_items()?;
    let display = board::display_name(&board_name);
    app.set_status(
        format!("Moved item {} to {}", id, display),
        StatusKind::Success,
    );
    Ok(())
}

fn set_priority(app: &mut App, id: u64, priority: u8) -> Result<()> {
    app.taskbook.update_priority_silent(id, priority)?;
    app.refresh_items()?;
    app.set_status(
        format!("Set priority {} for task {}", priority, id),
        StatusKind::Success,
    );
    Ok(())
}

fn delete_items(app: &mut App, ids: &[u64]) -> Result<()> {
    app.taskbook.delete_items_silent(ids)?;
    app.refresh_items()?;
    app.set_status(
        format!("Deleted {} item(s)", ids.len()),
        StatusKind::Success,
    );
    Ok(())
}

fn restore_item(app: &mut App, id: u64) -> Result<()> {
    app.taskbook.restore_items_silent(&[id])?;
    app.set_view(ViewMode::Archive)?;
    app.set_status(format!("Restored item {}", id), StatusKind::Success);
    Ok(())
}

fn copy_to_clipboard(app: &mut App, id: u64) -> Result<()> {
    app.taskbook.copy_to_clipboard_silent(&[id])?;
    app.set_status(
        format!("Copied item {} to clipboard", id),
        StatusKind::Success,
    );
    Ok(())
}

fn clear_completed(app: &mut App) -> Result<()> {
    let count = app.taskbook.clear_silent()?;
    app.refresh_items()?;
    app.set_status(
        format!("Cleared {} completed task(s)", count),
        StatusKind::Success,
    );
    Ok(())
}

fn rename_board(app: &mut App, old_name: &str, new_name: &str) -> Result<()> {
    let new_board = board::normalize_board_name(new_name);
    let count = app.taskbook.rename_board_silent(old_name, &new_board)?;

    if let Some(ref filter) = app.filter.board_filter {
        if board::board_eq(filter, old_name) {
            app.filter.board_filter = Some(new_board.clone());
        }
    }

    app.refresh_items()?;
    let old_display = board::display_name(old_name);
    let new_display = board::display_name(&new_board);
    app.set_status(
        format!(
            "Renamed {} to {} ({} items)",
            old_display, new_display, count
        ),
        StatusKind::Success,
    );
    Ok(())
}

fn edit_note_external(app: &mut App, id: u64) -> Result<()> {
    let item = app.items.get(&id.to_string());
    let note = match item.and_then(|i| i.as_note()) {
        Some(n) => n,
        None => {
            app.set_status("Item is not a note".to_string(), StatusKind::Error);
            return Ok(());
        }
    };

    let title = note.title().to_string();
    let body = note.body().map(|s| s.to_string());

    // Suspend TUI to run external editor
    let guard = super::suspend_tui()?;

    // Open external editor
    let content = editor::edit_existing_note_in_editor(&title, body.as_deref());

    // Resume TUI
    guard.resume()?;

    // After suspend/resume, ratatui's internal buffer is stale — force full redraw
    app.needs_full_redraw = true;

    match content? {
        Some(note_content) => {
            app.taskbook
                .edit_description_silent(id, &note_content.title)?;
            app.taskbook.edit_note_body_silent(id, note_content.body)?;
            app.refresh_items()?;
            app.set_status(format!("Updated note {}", id), StatusKind::Success);
        }
        None => {
            app.set_status("Edit cancelled".to_string(), StatusKind::Info);
        }
    }

    Ok(())
}
