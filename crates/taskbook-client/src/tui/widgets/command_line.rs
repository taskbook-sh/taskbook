use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};

use crate::tui::app::{App, PendingAction, SuggestionKind};

/// Render the command line at the bottom of the screen
pub fn render_command_line(frame: &mut Frame, app: &App, area: Rect) {
    // Pending confirmation takes over the command line
    if let Some(ref action) = app.command_line.pending_confirm {
        render_confirm(frame, app, area, action);
        return;
    }

    if app.command_line.focused {
        render_input(frame, app, area);
    } else {
        render_placeholder(frame, app, area);
    }
}

/// Render the unfocused command line with placeholder text
fn render_placeholder(frame: &mut Frame, _app: &App, area: Rect) {
    let prompt_style = Style::default().fg(Color::Rgb(80, 80, 100));
    let placeholder_style = Style::default().fg(Color::Rgb(100, 100, 120));

    let line = Line::from(vec![
        Span::styled("  > ", prompt_style),
        Span::styled("Type / or Tab for commands, ? for help", placeholder_style),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

/// Render the active input line
fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let input = &app.command_line.input;
    let cursor = app.command_line.cursor;
    let input_chars: Vec<char> = input.chars().collect();
    let char_count = input_chars.len();

    // Calculate visible window
    let usable_width = area.width.saturating_sub(4) as usize; // "  > " prefix
    let (display_start, display_end, cursor_in_display) = if char_count > usable_width {
        let start = cursor.saturating_sub(usable_width / 2);
        let end = (start + usable_width).min(char_count);
        let adjusted_start = if end == char_count {
            end.saturating_sub(usable_width)
        } else {
            start
        };
        (adjusted_start, end, cursor - adjusted_start)
    } else {
        (0, char_count, cursor)
    };

    let display_chars: String = input_chars[display_start..display_end].iter().collect();
    let cursor_pos = cursor_in_display.min(display_end - display_start);

    let before: String = display_chars.chars().take(cursor_pos).collect();
    let after_chars: Vec<char> = display_chars.chars().skip(cursor_pos).collect();
    let cursor_char = after_chars.first().copied().unwrap_or(' ');
    let after: String = after_chars.iter().skip(1).collect();

    let cursor_style = Style::default().bg(Color::White).fg(Color::Black);
    let prompt_style = app.theme.info.add_modifier(Modifier::BOLD);

    let line = Line::from(vec![
        Span::styled("  > ", prompt_style),
        Span::raw(before),
        Span::styled(cursor_char.to_string(), cursor_style),
        Span::raw(after),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

/// Render inline confirmation prompt
fn render_confirm(frame: &mut Frame, app: &App, area: Rect, action: &PendingAction) {
    let message = match action {
        PendingAction::Delete { ids } => {
            if ids.len() == 1 {
                format!("Delete item {}?", ids[0])
            } else {
                format!("Delete {} items?", ids.len())
            }
        }
        PendingAction::Clear => "Clear all completed tasks?".to_string(),
    };

    let bold = Style::default().add_modifier(Modifier::BOLD);

    let line = Line::from(vec![
        Span::raw("  "),
        Span::styled(&message, app.theme.warning),
        Span::raw("  "),
        Span::styled("[Enter]", bold),
        Span::raw(" Confirm  "),
        Span::styled("[Esc]", bold),
        Span::raw(" Cancel"),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

/// Render autocomplete dropdown floating above the command line
pub fn render_autocomplete(frame: &mut Frame, app: &App, content_area: Rect) {
    if app.command_line.suggestions.is_empty() || !app.command_line.focused {
        return;
    }

    let suggestions = &app.command_line.suggestions;
    let count = suggestions.len().min(8) as u16;

    // Position: bottom of content area, left-aligned with command line prompt
    let dropdown_height = count;
    let dropdown_y = content_area.y + content_area.height.saturating_sub(dropdown_height);
    let dropdown_x = content_area.x + 2; // align with prompt
    let dropdown_width = content_area.width.saturating_sub(4).min(50);

    let dropdown_area = Rect::new(dropdown_x, dropdown_y, dropdown_width, dropdown_height);

    // Clear the area behind the dropdown
    frame.render_widget(Clear, dropdown_area);

    let autocomplete_bg = Style::default().bg(Color::Rgb(40, 40, 55));
    let autocomplete_selected = Style::default()
        .bg(Color::Rgb(60, 60, 90))
        .add_modifier(Modifier::BOLD);
    let autocomplete_hint = Style::default()
        .fg(Color::Rgb(120, 120, 140))
        .bg(Color::Rgb(40, 40, 55));
    let autocomplete_hint_selected = Style::default()
        .fg(Color::Rgb(150, 150, 170))
        .bg(Color::Rgb(60, 60, 90));

    let selected = app.command_line.selected_suggestion;

    let mut lines: Vec<Line> = Vec::new();
    for (i, suggestion) in suggestions.iter().enumerate().take(count as usize) {
        let is_selected = selected == Some(i);
        let base_style = if is_selected {
            autocomplete_selected
        } else {
            autocomplete_bg
        };
        let hint_style = if is_selected {
            autocomplete_hint_selected
        } else {
            autocomplete_hint
        };

        let icon = match suggestion.kind {
            SuggestionKind::Command => "/",
            SuggestionKind::Board => "@",
            SuggestionKind::Item => "·",
        };

        let mut spans = vec![
            Span::styled(format!(" {} ", icon), hint_style),
            Span::styled(&suggestion.display, base_style),
        ];

        if let Some(ref desc) = suggestion.description {
            // Pad to align descriptions
            let name_len = suggestion.display.len() + 3; // icon + spaces
            let padding = dropdown_width as usize - name_len.min(dropdown_width as usize);
            let desc_max = padding.saturating_sub(2);
            if desc_max > 0 {
                let truncated: String = desc.chars().take(desc_max).collect();
                let pad_amount =
                    (dropdown_width as usize).saturating_sub(name_len + truncated.len() + 1);
                spans.push(Span::styled(" ".repeat(pad_amount), base_style));
                spans.push(Span::styled(truncated, hint_style));
            }
        }

        // Fill remaining width with background
        let line_len: usize = spans.iter().map(|s| s.content.len()).sum();
        if line_len < dropdown_width as usize {
            spans.push(Span::styled(
                " ".repeat(dropdown_width as usize - line_len),
                base_style,
            ));
        }

        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, dropdown_area);
}
