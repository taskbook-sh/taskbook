use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use ratatui::layout::Rect;

use crate::tui::app::{App, StatusKind, ViewMode};

/// Render the single-line stats/status bar
pub fn render_stats_line(frame: &mut Frame, app: &App, area: Rect) {
    // Status message takes priority
    if let Some(ref msg) = app.status_message {
        let style = match msg.kind {
            StatusKind::Success => app.theme.success,
            StatusKind::Error => app.theme.error,
            StatusKind::Info => app.theme.info,
        };
        let line = Line::from(vec![Span::raw("  "), Span::styled(&msg.text, style)]);
        frame.render_widget(Paragraph::new(line), area);
        return;
    }

    // Search indicator
    if let Some(ref term) = app.filter.search_term {
        let search_line = Line::from(vec![
            Span::raw("  "),
            Span::styled("Search: ", app.theme.info),
            Span::styled(
                format!("\"{}\"", term),
                app.theme.info.add_modifier(Modifier::BOLD),
            ),
            Span::styled("  (Esc to clear)", app.theme.muted),
        ]);
        frame.render_widget(Paragraph::new(search_line), area);
        return;
    }

    // Progress overview
    if app.config.display_progress_overview {
        let stats = app.get_stats();

        let mut spans = vec![
            Span::raw("  "),
            Span::styled(format!("{}%", stats.percent), app.theme.success),
            Span::styled(" done", app.theme.muted),
            Span::styled(" | ", app.theme.muted),
            Span::styled(format!("{}", stats.complete), app.theme.success),
            Span::styled(" done", app.theme.muted),
            Span::styled(" · ", app.theme.muted),
            Span::styled(format!("{}", stats.in_progress), app.theme.warning),
            Span::styled(" in-progress", app.theme.muted),
            Span::styled(" · ", app.theme.muted),
            Span::styled(format!("{}", stats.pending), app.theme.pending),
            Span::styled(" pending", app.theme.muted),
            Span::styled(" · ", app.theme.muted),
            Span::styled(format!("{}", stats.notes), app.theme.info),
            Span::styled(" notes", app.theme.muted),
        ];

        // Append key hints on the right
        append_key_hints(app, &mut spans);

        let stats_line = Line::from(spans);
        frame.render_widget(Paragraph::new(stats_line), area);
        return;
    }

    // No progress overview — show just key hints
    let mut spans = vec![Span::raw("  ")];
    append_key_hints(app, &mut spans);
    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line), area);
}

fn append_key_hints<'a>(app: &'a App, spans: &mut Vec<Span<'a>>) {
    let key_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let sep_style = app.theme.muted;

    spans.push(Span::styled("  ?", key_style));
    spans.push(Span::styled(" Help", sep_style));
    spans.push(Span::styled(" │ ", sep_style));
    spans.push(Span::styled("/", key_style));
    spans.push(Span::styled(" Command", sep_style));
    spans.push(Span::styled(" │ ", sep_style));

    if app.view == ViewMode::Archive {
        spans.push(Span::styled("r", key_style));
        spans.push(Span::styled(" Restore", sep_style));
    } else {
        spans.push(Span::styled("t", key_style));
        spans.push(Span::styled(" Task", sep_style));
        spans.push(Span::styled(" │ ", sep_style));
        spans.push(Span::styled("n", key_style));
        spans.push(Span::styled(" Note", sep_style));
    }

    spans.push(Span::styled(" │ ", sep_style));
    spans.push(Span::styled("q", key_style));
    spans.push(Span::styled(" Quit", sep_style));
}
