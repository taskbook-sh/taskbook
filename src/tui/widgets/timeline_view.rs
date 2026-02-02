use std::collections::HashMap;

use ratatui::{
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::models::StorageItem;
use crate::tui::app::App;

use super::item_row::{render_item_line, ItemRowOptions};

pub fn render_timeline_view(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    let mut item_line_map: Vec<Option<u64>> = Vec::new();
    let row_options = ItemRowOptions::for_timeline_view();

    // Group items by date
    let mut grouped: HashMap<String, Vec<&StorageItem>> = HashMap::new();
    for item in app.items.values() {
        let date = item.date().to_string();
        grouped.entry(date).or_default().push(item);
    }

    // Sort dates (newest first)
    let mut dates: Vec<String> = grouped.keys().cloned().collect();
    dates.sort_by(|a, b| {
        let items_a = grouped.get(a).unwrap();
        let items_b = grouped.get(b).unwrap();
        let ts_a = items_a.first().map(|i| i.timestamp()).unwrap_or(0);
        let ts_b = items_b.first().map(|i| i.timestamp()).unwrap_or(0);
        ts_b.cmp(&ts_a)
    });

    let today = chrono::Local::now().format("%a %b %d %Y").to_string();

    for date in dates {
        let date_items = grouped.get(&date).unwrap();

        // Count stats for this date (always count all tasks)
        let total_tasks: usize = date_items.iter().filter(|i| i.is_task()).count();
        let complete_tasks: usize = date_items.iter()
            .filter_map(|i| i.as_task())
            .filter(|t| t.is_complete)
            .count();

        // Filter items for display (respecting hide_completed)
        let visible_items: Vec<&StorageItem> = date_items.iter()
            .filter(|item| {
                if app.filter.hide_completed {
                    if let Some(task) = item.as_task() {
                        return !task.is_complete;
                    }
                }
                true
            })
            .copied()
            .collect();

        // Skip date if all visible items are hidden
        if visible_items.is_empty() {
            continue;
        }

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
        let mut sorted_items = visible_items;
        sorted_items.sort_by_key(|item| std::cmp::Reverse(item.timestamp()));

        for item in sorted_items {
            let is_selected = app.selected_id() == Some(item.id());
            let line = render_item_line(app, item, is_selected, &row_options);
            lines.push(line);
            item_line_map.push(Some(item.id()));
        }
    }

    render_scrollable_list(frame, area, lines, &item_line_map, app.selected_id());
}

fn render_scrollable_list(
    frame: &mut Frame,
    area: Rect,
    lines: Vec<Line<'static>>,
    item_line_map: &[Option<u64>],
    selected_id: Option<u64>,
) {
    let selected_line = item_line_map.iter()
        .position(|id| *id == selected_id)
        .unwrap_or(0);

    let scroll_offset = if selected_line >= area.height as usize {
        selected_line.saturating_sub(area.height as usize / 2)
    } else {
        0
    };

    let paragraph = Paragraph::new(lines.clone())
        .scroll((scroll_offset as u16, 0));
    frame.render_widget(paragraph, area);

    if lines.len() > area.height as usize {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None);
        let mut scrollbar_state = ScrollbarState::new(lines.len())
            .position(scroll_offset);
        frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    }
}
