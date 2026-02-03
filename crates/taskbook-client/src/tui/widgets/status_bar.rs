use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::tui::app::{App, StatusKind, ViewMode};

pub fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    // Stats line or status message
    if let Some(ref msg) = app.status_message {
        let style = match msg.kind {
            StatusKind::Success => app.theme.success,
            StatusKind::Error => app.theme.error,
            StatusKind::Info => app.theme.info,
        };
        let line = Line::from(vec![Span::raw("  "), Span::styled(&msg.text, style)]);
        frame.render_widget(Paragraph::new(line), chunks[0]);
    } else if app.config.display_progress_overview {
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
        frame.render_widget(Paragraph::new(stats_line), chunks[0]);
    }

    // Keybindings line
    let keybindings = match app.view {
        ViewMode::Board | ViewMode::Timeline => {
            vec![
                ("?", "Help"),
                ("t", "Task"),
                ("n", "Note"),
                ("c", "Check"),
                ("h", "Hide done"),
                ("q", "Quit"),
            ]
        }
        ViewMode::Archive => {
            vec![
                ("?", "Help"),
                ("r", "Restore"),
                ("1", "Board"),
                ("q", "Quit"),
            ]
        }
    };

    let mut spans: Vec<Span> = vec![Span::raw("  ")];
    for (i, (key, desc)) in keybindings.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", app.theme.muted));
        }
        spans.push(Span::styled(
            format!("[{}]", key),
            app.theme.muted.add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(format!(" {}", desc), app.theme.muted));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), chunks[1]);
}
