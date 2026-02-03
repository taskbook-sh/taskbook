use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::app::{App, PopupState, ViewMode};
use super::widgets::{
    board_picker::render_board_picker,
    board_view::render_board_view,
    help_popup::render_help_popup,
    input_dialog::render_input_dialog,
    status_bar::render_status_bar,
    timeline_view::render_timeline_view,
};

/// Render the entire UI
pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Header
            Constraint::Min(1),     // Content
            Constraint::Length(2),  // Status bar
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_content(frame, app, chunks[1]);
    render_status_bar(frame, app, chunks[2]);

    // Render popup if active
    if let Some(ref popup) = app.popup {
        render_popup(frame, app, popup);
    }
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let view_name = match app.view {
        ViewMode::Board => "Board View",
        ViewMode::Timeline => "Timeline View",
        ViewMode::Archive => "Archive View",
    };

    let mut spans = vec![
        Span::styled("  taskbook", app.theme.title),
        Span::raw("  "),
        Span::styled(view_name, app.theme.muted),
    ];

    // Show board filter indicator
    if let Some(ref board) = app.filter.board_filter {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(format!("[Filter: {}]", board), app.theme.info));
        spans.push(Span::styled(" (Esc to clear)", app.theme.muted));
    }

    // Show hide completed indicator
    if app.filter.hide_completed {
        spans.push(Span::raw("  "));
        spans.push(Span::styled("[Hiding completed]", app.theme.warning));
    }

    let header = Line::from(spans);
    let paragraph = Paragraph::new(header);
    frame.render_widget(paragraph, area);
}

fn render_content(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::TOP | Borders::BOTTOM)
        .border_style(app.theme.border);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.display_order.is_empty() {
        let empty_msg = match app.view {
            ViewMode::Board => "No tasks or notes. Press 't' to create a task or 'n' to create a note.",
            ViewMode::Timeline => "No tasks or notes.",
            ViewMode::Archive => "Archive is empty.",
        };
        let paragraph = Paragraph::new(empty_msg)
            .style(app.theme.muted)
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(paragraph, inner);
        return;
    }

    match app.view {
        ViewMode::Board => render_board_view(frame, app, inner),
        ViewMode::Timeline | ViewMode::Archive => render_timeline_view(frame, app, inner),
    }
}

fn render_popup(frame: &mut Frame, app: &App, popup: &PopupState) {
    match popup {
        PopupState::Help => render_help_popup(frame, app),
        PopupState::EditItem { input, cursor, .. } => {
            render_input_dialog(frame, app, "Edit Description", "Enter new description", input, *cursor);
        }
        PopupState::Search { input, cursor } => {
            render_input_dialog(frame, app, "Search", "Enter search term", input, *cursor);
        }
        PopupState::SelectBoardForMove { selected, .. } => {
            render_board_picker(frame, app, "Move to Board", *selected, true);
        }
        PopupState::SetPriority { id } => {
            render_priority_popup(frame, app, *id);
        }
        PopupState::ConfirmDelete { ids } => {
            render_confirm_popup(frame, app, &format!("Delete {} item(s)?", ids.len()));
        }
        PopupState::ConfirmClear => {
            render_confirm_popup(frame, app, "Delete all completed tasks?");
        }
        PopupState::SelectBoardForTask { selected } => {
            render_board_picker(frame, app, "Select Board for Task", *selected, true);
        }
        PopupState::SelectBoardForNote { selected } => {
            render_board_picker(frame, app, "Select Board for Note", *selected, true);
        }
        PopupState::CreateBoard { input, cursor } => {
            render_input_dialog(frame, app, "Create Board", "Enter board name (without @)", input, *cursor);
        }
        PopupState::CreateTaskWithBoard { board, input, cursor } => {
            render_input_dialog(frame, app, &format!("Create Task in {}", board), "description p:1-3", input, *cursor);
        }
        PopupState::CreateNoteWithBoard { board, input, cursor } => {
            render_input_dialog(frame, app, &format!("Create Note in {}", board), "description", input, *cursor);
        }
        PopupState::RenameBoard { old_name, input, cursor } => {
            render_input_dialog(frame, app, &format!("Rename {}", old_name), "new board name (without @)", input, *cursor);
        }
    }
}

fn render_priority_popup(frame: &mut Frame, app: &App, _id: u64) {
    let area = centered_rect(30, 9, frame.area());

    let block = Block::default()
        .title(" Set Priority ")
        .borders(Borders::ALL)
        .border_style(app.theme.border)
        .style(Style::default().bg(ratatui::style::Color::Black));

    let inner = block.inner(area);
    frame.render_widget(ratatui::widgets::Clear, area);
    frame.render_widget(block, area);

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("1", app.theme.muted),
            Span::raw(" - Normal"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("2", app.theme.warning),
            Span::raw(" - Medium "),
            Span::styled("(!)", app.theme.warning),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("3", app.theme.error),
            Span::raw(" - High "),
            Span::styled("(!!)", app.theme.error),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Press 1, 2, or 3", app.theme.muted)),
    ];

    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, inner);
}

fn render_confirm_popup(frame: &mut Frame, app: &App, message: &str) {
    let area = centered_rect(40, 7, frame.area());

    let block = Block::default()
        .title(" Confirm ")
        .borders(Borders::ALL)
        .border_style(app.theme.border)
        .style(Style::default().bg(ratatui::style::Color::Black));

    let inner = block.inner(area);
    frame.render_widget(ratatui::widgets::Clear, area);
    frame.render_widget(block, area);

    let text = vec![
        Line::from(""),
        Line::from(Span::raw(format!("  {}", message))),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("[Enter]", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Confirm  "),
            Span::styled("[Esc]", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Cancel"),
        ]),
    ];

    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, inner);
}

/// Helper function to create a centered rect
pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
