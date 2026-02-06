use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::tui::ui::centered_rect;

pub fn render_input_dialog(
    frame: &mut Frame,
    app: &App,
    title: &str,
    hint: &str,
    input: &str,
    cursor: usize,
) {
    let width = 50.min(frame.area().width.saturating_sub(4));
    let area = centered_rect(width, 8, frame.area());

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(app.theme.border)
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    // Input field with cursor (handle UTF-8 properly)
    let input_width = inner.width.saturating_sub(4) as usize;
    let input_chars: Vec<char> = input.chars().collect();
    let char_count = input_chars.len();

    // Calculate visible window of characters
    let (display_start, display_end, cursor_in_display) = if char_count > input_width {
        let start = cursor.saturating_sub(input_width / 2);
        let end = (start + input_width).min(char_count);
        let adjusted_start = if end == char_count {
            end.saturating_sub(input_width)
        } else {
            start
        };
        (adjusted_start, end, cursor - adjusted_start)
    } else {
        (0, char_count, cursor)
    };

    let display_chars: String = input_chars[display_start..display_end].iter().collect();
    let cursor_pos = cursor_in_display.min(display_end - display_start);

    // Split at cursor position (character-based)
    let before: String = display_chars.chars().take(cursor_pos).collect();
    let after_chars: Vec<char> = display_chars.chars().skip(cursor_pos).collect();
    let cursor_char = after_chars.first().copied().unwrap_or(' ');
    let after: String = after_chars.iter().skip(1).collect();

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::raw(before.clone()),
            Span::styled(
                cursor_char.to_string(),
                Style::default().bg(Color::White).fg(Color::Black),
            ),
            Span::raw(after.clone()),
        ]),
        Line::from(""),
        Line::from(Span::styled(format!("  Syntax: {}", hint), app.theme.muted)),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("[Enter]", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Confirm  "),
            Span::styled("[Esc]", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Cancel"),
        ]),
    ];

    frame.render_widget(Paragraph::new(text), inner);
}
