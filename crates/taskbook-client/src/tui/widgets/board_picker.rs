use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::tui::ui::centered_rect;
use taskbook_common::board;

pub fn render_board_picker(
    frame: &mut Frame,
    app: &App,
    title: &str,
    selected: usize,
    show_new_board_option: bool,
) {
    let items_count = app.boards.len() + if show_new_board_option { 1 } else { 0 };
    let height = (items_count + 4).min(20) as u16;
    let area = centered_rect(40, height, frame.area());

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(app.theme.border)
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    // List existing boards
    for (i, b) in app.boards.iter().enumerate() {
        let is_selected = i == selected;
        let prefix = if is_selected { " > " } else { "   " };
        let style = if is_selected {
            app.theme.selected.add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let display = board::display_name(b);
        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(display, style),
        ]));
    }

    // "New board" option
    if show_new_board_option {
        let is_selected = selected == app.boards.len();
        let prefix = if is_selected { " > " } else { "   " };
        let style = if is_selected {
            app.theme.info.add_modifier(Modifier::BOLD)
        } else {
            app.theme.info
        };
        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled("+ New board...", style),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("[Enter]", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Select  "),
        Span::styled("[Esc]", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Cancel"),
    ]));

    frame.render_widget(Paragraph::new(lines), inner);
}
