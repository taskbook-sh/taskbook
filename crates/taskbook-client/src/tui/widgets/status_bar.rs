use ratatui::{
    style::Modifier,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use ratatui::layout::Rect;

use crate::tui::app::{App, StatusKind};

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
        let stats_line = Line::from(vec![
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
        ]);
        frame.render_widget(Paragraph::new(stats_line), area);
    }
}
