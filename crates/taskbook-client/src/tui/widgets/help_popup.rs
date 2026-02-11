use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::tui::ui::centered_rect;

pub fn render_help_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 32, frame.area());

    let block = Block::default()
        .title(" Keybindings ")
        .borders(Borders::ALL)
        .border_style(app.theme.border)
        .style(Style::default().bg(Color::Black));

    let key_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let desc_style = app.theme.muted;
    let section_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled("  Navigation", section_style)),
        Line::from(vec![
            Span::styled("    j/↓     ", key_style),
            Span::styled("Move down", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    k/↑     ", key_style),
            Span::styled("Move up", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    g       ", key_style),
            Span::styled("Go to top", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    G       ", key_style),
            Span::styled("Go to bottom", desc_style),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Actions", section_style)),
        Line::from(vec![
            Span::styled("    t       ", key_style),
            Span::styled("Create task", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    n       ", key_style),
            Span::styled("Create note", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    c       ", key_style),
            Span::styled("Toggle check", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    b       ", key_style),
            Span::styled("Toggle in-progress", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    s       ", key_style),
            Span::styled("Toggle star", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    e       ", key_style),
            Span::styled("Edit description", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    m       ", key_style),
            Span::styled("Move to board", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    p       ", key_style),
            Span::styled("Set priority", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    d       ", key_style),
            Span::styled("Delete item", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    r       ", key_style),
            Span::styled("Restore (archive)", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    y       ", key_style),
            Span::styled("Copy to clipboard", desc_style),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Views & Filters", section_style)),
        Line::from(vec![
            Span::styled("    1       ", key_style),
            Span::styled("Board view", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    2       ", key_style),
            Span::styled("Timeline view", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    3       ", key_style),
            Span::styled("Archive view", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    /       ", key_style),
            Span::styled("Search items", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    h       ", key_style),
            Span::styled("Toggle hide completed", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    Esc     ", key_style),
            Span::styled("Clear search/filter / Quit", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    Enter   ", key_style),
            Span::styled("Filter board / Edit note", desc_style),
        ]),
        Line::from(""),
        Line::from(Span::styled("        Press any key to close", desc_style)),
    ];

    frame.render_widget(Clear, area);
    frame.render_widget(block.clone(), area);
    let inner = block.inner(area);
    frame.render_widget(Paragraph::new(text), inner);
}
