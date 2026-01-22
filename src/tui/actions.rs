use crossterm::event::{KeyCode, KeyEvent};

use crate::error::Result;

use super::app::{App, PopupState, StatusKind, ViewMode};
use super::input_handler::{handle_text_input, normalize_board_name, InputResult};

/// Handle a key event
pub fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<()> {
    // Handle popup-specific keys first
    if let Some(popup) = app.popup.clone() {
        return handle_popup_key(app, key, popup);
    }

    match key.code {
        // Quit
        KeyCode::Char('q') => app.quit(),
        KeyCode::Esc => {
            // If board filter is active, clear it; otherwise quit
            if app.filter.board_filter.is_some() {
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

        // Enter to filter by board (in board view)
        KeyCode::Enter if app.view == ViewMode::Board && app.filter.board_filter.is_none() => {
            if let Some(board) = app.get_board_for_selected() {
                app.set_board_filter(Some(board.clone()));
                app.set_status(format!("Filtering by {}", board), StatusKind::Info);
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

        // Help
        KeyCode::Char('?') => {
            app.popup = Some(PopupState::Help);
        }

        // Create task - skip board picker if already filtering
        KeyCode::Char('t') if app.view != ViewMode::Archive => {
            if let Some(ref board) = app.filter.board_filter {
                app.popup = Some(PopupState::CreateTaskWithBoard {
                    board: board.clone(),
                    input: String::new(),
                    cursor: 0,
                });
            } else {
                app.popup = Some(PopupState::SelectBoardForTask { selected: 0 });
            }
        }
        // Create note - skip board picker if already filtering
        KeyCode::Char('n') if app.view != ViewMode::Archive => {
            if let Some(ref board) = app.filter.board_filter {
                app.popup = Some(PopupState::CreateNoteWithBoard {
                    board: board.clone(),
                    input: String::new(),
                    cursor: 0,
                });
            } else {
                app.popup = Some(PopupState::SelectBoardForNote { selected: 0 });
            }
        }
        // Create new board
        KeyCode::Char('B') if app.view != ViewMode::Archive => {
            app.popup = Some(PopupState::CreateBoard {
                input: String::new(),
                cursor: 0,
            });
        }
        // Rename board
        KeyCode::Char('R') if app.view == ViewMode::Board => {
            let board = app.filter.board_filter.clone()
                .or_else(|| app.get_board_for_selected());
            if let Some(board_name) = board {
                app.popup = Some(PopupState::RenameBoard {
                    old_name: board_name,
                    input: String::new(),
                    cursor: 0,
                });
            }
        }

        // Item actions (require selection)
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
        KeyCode::Char('e') if app.view != ViewMode::Archive => {
            if let Some(item) = app.selected_item() {
                let id = item.id();
                let desc = item.description().to_string();
                let cursor = desc.chars().count();
                app.popup = Some(PopupState::EditItem {
                    id,
                    input: desc,
                    cursor,
                });
            }
        }
        KeyCode::Char('m') if app.view != ViewMode::Archive => {
            if let Some(id) = app.selected_id() {
                app.popup = Some(PopupState::MoveBoard {
                    id,
                    input: String::new(),
                    cursor: 0,
                });
            }
        }
        KeyCode::Char('p') if app.view != ViewMode::Archive => {
            if let Some(item) = app.selected_item() {
                if item.is_task() {
                    app.popup = Some(PopupState::SetPriority { id: item.id() });
                }
            }
        }
        KeyCode::Char('d') if app.view != ViewMode::Archive => {
            if let Some(id) = app.selected_id() {
                app.popup = Some(PopupState::ConfirmDelete { ids: vec![id] });
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
        KeyCode::Char('C') if app.view != ViewMode::Archive => {
            app.popup = Some(PopupState::ConfirmClear);
        }
        KeyCode::Char('/') => {
            app.popup = Some(PopupState::Search {
                input: String::new(),
                cursor: 0,
            });
        }

        _ => {}
    }

    Ok(())
}

fn handle_popup_key(app: &mut App, key: KeyEvent, popup: PopupState) -> Result<()> {
    match popup {
        PopupState::Help => {
            app.popup = None;
        }
        PopupState::EditItem { id, input, cursor } => {
            match handle_text_input(key, &input, cursor) {
                InputResult::Cancel => app.popup = None,
                InputResult::Submit => {
                    if !input.trim().is_empty() {
                        edit_description(app, id, &input)?;
                    }
                    app.popup = None;
                }
                InputResult::Changed { input: new_input, cursor: new_cursor } => {
                    app.popup = Some(PopupState::EditItem {
                        id,
                        input: new_input,
                        cursor: new_cursor,
                    });
                }
                InputResult::Ignored => {
                    app.popup = Some(PopupState::EditItem { id, input, cursor });
                }
            }
        }
        PopupState::Search { input, cursor } => {
            match handle_text_input(key, &input, cursor) {
                InputResult::Cancel => {
                    app.popup = None;
                    app.filter.search_term = None;
                    app.refresh_items()?;
                }
                InputResult::Submit => {
                    if !input.trim().is_empty() {
                        app.filter.search_term = Some(input);
                    }
                    app.popup = None;
                }
                InputResult::Changed { input: new_input, cursor: new_cursor } => {
                    app.popup = Some(PopupState::Search {
                        input: new_input,
                        cursor: new_cursor,
                    });
                }
                InputResult::Ignored => {
                    app.popup = Some(PopupState::Search { input, cursor });
                }
            }
        }
        PopupState::MoveBoard { id, input, cursor } => {
            match handle_text_input(key, &input, cursor) {
                InputResult::Cancel => app.popup = None,
                InputResult::Submit => {
                    if !input.trim().is_empty() {
                        move_to_board(app, id, &input)?;
                    }
                    app.popup = None;
                }
                InputResult::Changed { input: new_input, cursor: new_cursor } => {
                    app.popup = Some(PopupState::MoveBoard {
                        id,
                        input: new_input,
                        cursor: new_cursor,
                    });
                }
                InputResult::Ignored => {
                    app.popup = Some(PopupState::MoveBoard { id, input, cursor });
                }
            }
        }
        PopupState::SetPriority { id } => {
            match key.code {
                KeyCode::Esc => app.popup = None,
                KeyCode::Char('1') => {
                    set_priority(app, id, 1)?;
                    app.popup = None;
                }
                KeyCode::Char('2') => {
                    set_priority(app, id, 2)?;
                    app.popup = None;
                }
                KeyCode::Char('3') => {
                    set_priority(app, id, 3)?;
                    app.popup = None;
                }
                _ => {}
            }
        }
        PopupState::ConfirmDelete { ids } => {
            match key.code {
                KeyCode::Esc => app.popup = None,
                KeyCode::Enter => {
                    delete_items(app, &ids)?;
                    app.popup = None;
                }
                _ => {}
            }
        }
        PopupState::ConfirmClear => {
            match key.code {
                KeyCode::Esc => app.popup = None,
                KeyCode::Enter => {
                    clear_completed(app)?;
                    app.popup = None;
                }
                _ => {}
            }
        }
        PopupState::SelectBoardForTask { mut selected } => {
            let max_index = app.boards.len();
            match key.code {
                KeyCode::Esc => app.popup = None,
                KeyCode::Char('j') | KeyCode::Down => {
                    if selected < max_index {
                        selected += 1;
                    }
                    app.popup = Some(PopupState::SelectBoardForTask { selected });
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    selected = selected.saturating_sub(1);
                    app.popup = Some(PopupState::SelectBoardForTask { selected });
                }
                KeyCode::Enter => {
                    if selected < app.boards.len() {
                        let board = app.boards[selected].clone();
                        app.popup = Some(PopupState::CreateTaskWithBoard {
                            board,
                            input: String::new(),
                            cursor: 0,
                        });
                    } else {
                        app.popup = Some(PopupState::CreateBoard {
                            input: String::new(),
                            cursor: 0,
                        });
                    }
                }
                _ => {}
            }
        }
        PopupState::SelectBoardForNote { mut selected } => {
            let max_index = app.boards.len();
            match key.code {
                KeyCode::Esc => app.popup = None,
                KeyCode::Char('j') | KeyCode::Down => {
                    if selected < max_index {
                        selected += 1;
                    }
                    app.popup = Some(PopupState::SelectBoardForNote { selected });
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    selected = selected.saturating_sub(1);
                    app.popup = Some(PopupState::SelectBoardForNote { selected });
                }
                KeyCode::Enter => {
                    if selected < app.boards.len() {
                        let board = app.boards[selected].clone();
                        app.popup = Some(PopupState::CreateNoteWithBoard {
                            board,
                            input: String::new(),
                            cursor: 0,
                        });
                    } else {
                        app.popup = Some(PopupState::CreateBoard {
                            input: String::new(),
                            cursor: 0,
                        });
                    }
                }
                _ => {}
            }
        }
        PopupState::CreateBoard { input, cursor } => {
            match handle_text_input(key, &input, cursor) {
                InputResult::Cancel => app.popup = None,
                InputResult::Submit => {
                    if !input.trim().is_empty() {
                        let board_name = normalize_board_name(&input);
                        app.refresh_items()?;
                        if !app.boards.contains(&board_name) {
                            app.boards.push(board_name.clone());
                        }
                        app.set_status(format!("Board {} ready - add a task or note to it", board_name), StatusKind::Success);
                    }
                    app.popup = None;
                }
                InputResult::Changed { input: new_input, cursor: new_cursor } => {
                    app.popup = Some(PopupState::CreateBoard {
                        input: new_input,
                        cursor: new_cursor,
                    });
                }
                InputResult::Ignored => {
                    app.popup = Some(PopupState::CreateBoard { input, cursor });
                }
            }
        }
        PopupState::CreateTaskWithBoard { board, input, cursor } => {
            match handle_text_input(key, &input, cursor) {
                InputResult::Cancel => app.popup = None,
                InputResult::Submit => {
                    if !input.trim().is_empty() {
                        create_task_in_board(app, &board, &input)?;
                    }
                    app.popup = None;
                }
                InputResult::Changed { input: new_input, cursor: new_cursor } => {
                    app.popup = Some(PopupState::CreateTaskWithBoard {
                        board,
                        input: new_input,
                        cursor: new_cursor,
                    });
                }
                InputResult::Ignored => {
                    app.popup = Some(PopupState::CreateTaskWithBoard { board, input, cursor });
                }
            }
        }
        PopupState::CreateNoteWithBoard { board, input, cursor } => {
            match handle_text_input(key, &input, cursor) {
                InputResult::Cancel => app.popup = None,
                InputResult::Submit => {
                    if !input.trim().is_empty() {
                        create_note_in_board(app, &board, &input)?;
                    }
                    app.popup = None;
                }
                InputResult::Changed { input: new_input, cursor: new_cursor } => {
                    app.popup = Some(PopupState::CreateNoteWithBoard {
                        board,
                        input: new_input,
                        cursor: new_cursor,
                    });
                }
                InputResult::Ignored => {
                    app.popup = Some(PopupState::CreateNoteWithBoard { board, input, cursor });
                }
            }
        }
        PopupState::RenameBoard { old_name, input, cursor } => {
            match handle_text_input(key, &input, cursor) {
                InputResult::Cancel => app.popup = None,
                InputResult::Submit => {
                    if !input.trim().is_empty() {
                        rename_board(app, &old_name, &input)?;
                    }
                    app.popup = None;
                }
                InputResult::Changed { input: new_input, cursor: new_cursor } => {
                    app.popup = Some(PopupState::RenameBoard {
                        old_name,
                        input: new_input,
                        cursor: new_cursor,
                    });
                }
                InputResult::Ignored => {
                    app.popup = Some(PopupState::RenameBoard { old_name, input, cursor });
                }
            }
        }
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
            app.set_status(format!("Toggled in-progress for task {}", id), StatusKind::Success);
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
    let board_name = normalize_board_name(board);
    app.taskbook.move_boards_silent(id, vec![board_name.clone()])?;
    app.refresh_items()?;
    app.set_status(format!("Moved item {} to {}", id, board_name), StatusKind::Success);
    Ok(())
}

fn set_priority(app: &mut App, id: u64, priority: u8) -> Result<()> {
    app.taskbook.update_priority_silent(id, priority)?;
    app.refresh_items()?;
    app.set_status(format!("Set priority {} for task {}", priority, id), StatusKind::Success);
    Ok(())
}

fn delete_items(app: &mut App, ids: &[u64]) -> Result<()> {
    app.taskbook.delete_items_silent(ids)?;
    app.refresh_items()?;
    app.set_status(format!("Deleted {} item(s)", ids.len()), StatusKind::Success);
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
    app.set_status(format!("Copied item {} to clipboard", id), StatusKind::Success);
    Ok(())
}

fn clear_completed(app: &mut App) -> Result<()> {
    let count = app.taskbook.clear_silent()?;
    app.refresh_items()?;
    app.set_status(format!("Cleared {} completed task(s)", count), StatusKind::Success);
    Ok(())
}

fn create_task_in_board(app: &mut App, board: &str, input: &str) -> Result<()> {
    let full_input = format!("{} {}", board, input);
    let parts: Vec<String> = full_input.split_whitespace().map(String::from).collect();
    app.taskbook.create_task_silent(&parts)?;
    app.refresh_items()?;
    app.set_status(format!("Task created in {}", board), StatusKind::Success);
    Ok(())
}

fn create_note_in_board(app: &mut App, board: &str, input: &str) -> Result<()> {
    let full_input = format!("{} {}", board, input);
    let parts: Vec<String> = full_input.split_whitespace().map(String::from).collect();
    app.taskbook.create_note_silent(&parts)?;
    app.refresh_items()?;
    app.set_status(format!("Note created in {}", board), StatusKind::Success);
    Ok(())
}

fn rename_board(app: &mut App, old_name: &str, new_name: &str) -> Result<()> {
    let new_board = normalize_board_name(new_name);
    let count = app.taskbook.rename_board_silent(old_name, &new_board)?;

    if app.filter.board_filter.as_ref() == Some(&old_name.to_string()) {
        app.filter.board_filter = Some(new_board.clone());
    }

    app.refresh_items()?;
    app.set_status(format!("Renamed {} to {} ({} items)", old_name, new_board, count), StatusKind::Success);
    Ok(())
}
