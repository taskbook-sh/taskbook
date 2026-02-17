use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::app::{App, PopupState, ViewMode};
use super::widgets::{
    board_view::render_board_view, command_line::render_autocomplete,
    command_line::render_command_line, help_popup::render_help_popup,
    journal_view::render_journal_view, status_bar::render_stats_line,
    timeline_view::render_timeline_view,
};

/// Render the entire UI
pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Min(1),    // Content
            Constraint::Length(1), // Stats line
            Constraint::Length(1), // Command line
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_content(frame, app, chunks[1]);
    render_stats_line(frame, app, chunks[2]);
    render_command_line(frame, app, chunks[3]);

    // Render autocomplete overlay on top of content area
    render_autocomplete(frame, app, chunks[1]);

    // Render popup if active (Help only)
    if let Some(ref popup) = app.popup {
        render_popup(frame, app, popup);
    }
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let view_name = match app.view {
        ViewMode::Board => "Board View",
        ViewMode::Timeline => "Timeline View",
        ViewMode::Archive => "Archive View",
        ViewMode::Journal => "Journal View",
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
            ViewMode::Board => {
                "No tasks or notes. Press 't' to create a task or 'n' to create a note."
            }
            ViewMode::Timeline => "No tasks or notes.",
            ViewMode::Journal => "Journal is empty.",
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
        ViewMode::Journal => render_journal_view(frame, app, inner),
    }
}

fn render_popup(frame: &mut Frame, app: &App, popup: &PopupState) {
    match popup {
        PopupState::Help => render_help_popup(frame, app),
    }
}

/// Helper function to create a centered rect
pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
