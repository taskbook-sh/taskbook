use std::collections::HashMap;

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::models::StorageItem;
use crate::tui::app::App;

pub fn render_timeline_view(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    let mut item_line_map: Vec<Option<u64>> = Vec::new();

    // Group items by date
    let mut grouped: HashMap<String, Vec<&StorageItem>> = HashMap::new();
    for item in app.items.values() {
        let date = item.date().to_string();
        grouped.entry(date).or_default().push(item);
    }

    // Sort dates (newest first)
    let mut dates: Vec<String> = grouped.keys().cloned().collect();
    dates.sort_by(|a, b| {
        // Parse dates and compare
        let items_a = grouped.get(a).unwrap();
        let items_b = grouped.get(b).unwrap();
        let ts_a = items_a.first().map(|i| i.timestamp()).unwrap_or(0);
        let ts_b = items_b.first().map(|i| i.timestamp()).unwrap_or(0);
        ts_b.cmp(&ts_a)
    });

    let today = chrono::Local::now().format("%a %b %d %Y").to_string();

    for date in dates {
        let date_items = grouped.get(&date).unwrap();

        // Count stats for this date
        let total_tasks: usize = date_items.iter().filter(|i| i.is_task()).count();
        let complete_tasks: usize = date_items.iter()
            .filter_map(|i| i.as_task())
            .filter(|t| t.is_complete)
            .count();

        // Date header
        lines.push(Line::from(""));
        item_line_map.push(None);

        let is_today = date == today;
        let date_header = if total_tasks > 0 {
            if is_today {
                format!("  {} [Today] [{}/{}]", date, complete_tasks, total_tasks)
            } else {
                format!("  {} [{}/{}]", date, complete_tasks, total_tasks)
            }
        } else if is_today {
            format!("  {} [Today]", date)
        } else {
            format!("  {}", date)
        };

        let header_style = if is_today {
            app.theme.header.add_modifier(Modifier::BOLD)
        } else {
            app.theme.header
        };
        lines.push(Line::from(Span::styled(date_header, header_style)));
        item_line_map.push(None);

        // Sort items by timestamp (newest first)
        let mut sorted_items: Vec<&StorageItem> = date_items.clone();
        sorted_items.sort_by(|a, b| b.timestamp().cmp(&a.timestamp()));

        for item in sorted_items {
            let is_selected = app.selected_id() == Some(item.id());
            let line = render_item_line(app, item, is_selected);
            lines.push(line);
            item_line_map.push(Some(item.id()));
        }
    }

    // Calculate scroll position
    let selected_line = item_line_map.iter()
        .position(|id| *id == app.selected_id())
        .unwrap_or(0);

    let scroll_offset = if selected_line >= area.height as usize {
        selected_line.saturating_sub(area.height as usize / 2)
    } else {
        0
    };

    let paragraph = Paragraph::new(lines.clone())
        .scroll((scroll_offset as u16, 0));
    frame.render_widget(paragraph, area);

    // Scrollbar
    if lines.len() > area.height as usize {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None);
        let mut scrollbar_state = ScrollbarState::new(lines.len())
            .position(scroll_offset);
        frame.render_stateful_widget(
            scrollbar,
            area,
            &mut scrollbar_state,
        );
    }
}

fn render_item_line(app: &App, item: &StorageItem, is_selected: bool) -> Line<'static> {
    let mut spans: Vec<Span> = Vec::new();

    // Selection indicator
    if is_selected {
        spans.push(Span::styled(" > ", app.theme.info));
    } else {
        spans.push(Span::raw("   "));
    }

    // ID - use brighter style
    spans.push(Span::styled(
        format!("{:3}. ", item.id()),
        app.theme.item_id,
    ));

    // Icon
    let (icon, icon_style) = if let Some(task) = item.as_task() {
        if task.is_complete {
            ("✔", app.theme.success)
        } else if task.in_progress {
            ("…", app.theme.warning)
        } else {
            ("☐", app.theme.pending)
        }
    } else {
        ("●", app.theme.info)
    };
    spans.push(Span::styled(format!("{} ", icon), icon_style));

    // Description - better styling for completed tasks
    let desc = item.description().to_string();
    let desc_style = if let Some(task) = item.as_task() {
        if task.is_complete {
            // Use success color (green) with dim modifier - more readable than strikethrough
            app.theme.completed_text
        } else if task.priority == 3 {
            app.theme.error.add_modifier(Modifier::BOLD)
        } else if task.priority == 2 {
            app.theme.warning
        } else {
            Style::default().fg(Color::White)
        }
    } else {
        Style::default().fg(Color::Rgb(200, 200, 220))
    };
    spans.push(Span::styled(desc, desc_style));

    // Boards
    let boards = item.boards().join(" ");
    if !boards.is_empty() {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(boards, app.theme.muted));
    }

    // Priority indicator
    if let Some(task) = item.as_task() {
        if task.priority == 2 {
            spans.push(Span::styled(" (!)", app.theme.warning));
        } else if task.priority == 3 {
            spans.push(Span::styled(" (!!)", app.theme.error));
        }
    }

    // Star
    if item.is_starred() {
        spans.push(Span::styled(" ★", app.theme.starred));
    }

    let mut line = Line::from(spans);
    if is_selected {
        line = line.style(app.theme.selected);
    }
    line
}
