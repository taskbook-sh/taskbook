use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::tui::ui::centered_rect;

pub fn render_help_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(56, 38, frame.area());

    let block = Block::default()
        .title(" Keybindings & Commands ")
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
    let cmd_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled("  Navigation", section_style)),
        Line::from(vec![
            Span::styled("    j/k ↑/↓  ", key_style),
            Span::styled("Move up/down", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    g/G      ", key_style),
            Span::styled("Go to top/bottom", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    Enter    ", key_style),
            Span::styled("Filter board / Edit note", desc_style),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Quick Actions", section_style)),
        Line::from(vec![
            Span::styled("    c        ", key_style),
            Span::styled("Toggle check", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    b        ", key_style),
            Span::styled("Toggle in-progress", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    s        ", key_style),
            Span::styled("Toggle star", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    y        ", key_style),
            Span::styled("Copy to clipboard", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    1-4      ", key_style),
            Span::styled("Switch view (board/time/arch/jour)", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    h        ", key_style),
            Span::styled("Toggle hide completed", desc_style),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Command Line Shortcuts", section_style)),
        Line::from(vec![
            Span::styled("    t        ", key_style),
            Span::styled("→ /task @...", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    n        ", key_style),
            Span::styled("→ /note @...", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    e        ", key_style),
            Span::styled("→ /edit @<id> <desc>", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    m        ", key_style),
            Span::styled("→ /move @<id> @...", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    p        ", key_style),
            Span::styled("→ /priority @<id> ...", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    d        ", key_style),
            Span::styled("Delete selected (confirm)", desc_style),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Slash Commands", section_style)),
        Line::from(vec![
            Span::styled("    /task    ", cmd_style),
            Span::styled("@board Description p:2", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    /note    ", cmd_style),
            Span::styled("@board Title", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    /edit    ", cmd_style),
            Span::styled("@<id> New description", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    /move    ", cmd_style),
            Span::styled("@<id> @board", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    /delete  ", cmd_style),
            Span::styled("<id> [id...]", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    /search  ", cmd_style),
            Span::styled("<term>", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    /clear   ", cmd_style),
            Span::styled("Clear completed tasks", desc_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    Tab      ", key_style),
            Span::styled("Accept suggestion", desc_style),
        ]),
        Line::from(""),
        Line::from(Span::styled("        Press any key to close", desc_style)),
    ];

    frame.render_widget(Clear, area);
    frame.render_widget(block.clone(), area);
    let inner = block.inner(area);
    frame.render_widget(Paragraph::new(text), inner);
}
