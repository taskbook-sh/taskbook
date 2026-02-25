use std::collections::HashMap;

use chrono::{Local, TimeZone};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::tui::app::App;
use taskbook_common::StorageItem;

pub fn render_journal_view(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    let mut item_line_map: Vec<Option<u64>> = Vec::new();

    // Group items by date
    let mut grouped: HashMap<String, Vec<&StorageItem>> = HashMap::new();
    for item in app.items.values() {
        let date = item.date().to_string();
        grouped.entry(date).or_default().push(item);
    }

    // Sort dates (oldest first - chronological for journal)
    let mut dates: Vec<String> = grouped.keys().cloned().collect();
    dates.sort_by(|a, b| {
        let items_a = grouped.get(a).unwrap();
        let items_b = grouped.get(b).unwrap();
        // Use the maximum timestamp in the group to represent the group date
        let ts_a = items_a.iter().map(|i| i.timestamp()).max().unwrap_or(0);
        let ts_b = items_b.iter().map(|i| i.timestamp()).max().unwrap_or(0);
        ts_b.cmp(&ts_a) // Newest first
    });

    let today = chrono::Local::now().format("%a %b %d %Y").to_string();

    let mut first_group = true;
    for date in dates {
        let date_items = grouped.get(&date).unwrap();

        // Filter items for display - journal always shows completed tasks
        let visible_items: Vec<&StorageItem> = date_items
            .iter()
            .filter(|item| {
                // Only apply search filter, skip hide_completed
                if let Some(ref term) = app.filter.search_term {
                    let term_lower = term.to_lowercase();
                    if !item.description().to_lowercase().contains(&term_lower) {
                        return false;
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

        // Date header (blank separator between groups, not before first)
        if !first_group {
            lines.push(Line::from(""));
            item_line_map.push(None);
        }
        first_group = false;

        let is_today = date == today;
        let date_header = if is_today {
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

        // Sort items by timestamp (newest first), then by ID (asc) to match display order
        let mut sorted_items = visible_items;
        sorted_items.sort_by(|a, b| {
            b.timestamp()
                .cmp(&a.timestamp())
                .then_with(|| a.id().cmp(&b.id()))
        });

        for item in sorted_items {
            let is_selected = app.selected_id() == Some(item.id());

            // Format time
            let time_str = Local
                .timestamp_millis_opt(item.timestamp())
                .single()
                .map(|dt| dt.format("%H:%M").to_string())
                .unwrap_or_else(|| "??:??".to_string());

            let time_span = Span::styled(format!("  {} ", time_str), app.theme.muted);

            // Title/Description
            let desc_style = if is_selected {
                app.theme.selected.add_modifier(Modifier::BOLD)
            } else if let Some(task) = item.as_task() {
                if task.is_complete {
                    app.theme
                        .completed_text
                        .remove_modifier(Modifier::CROSSED_OUT)
                } else if task.in_progress {
                    app.theme.warning
                } else {
                    Style::default().fg(Color::White)
                }
            } else {
                // Note title
                Style::default().fg(Color::Rgb(200, 200, 220))
            };

            let mut title_spans = vec![time_span];

            // Add icon for tasks
            if let Some(task) = item.as_task() {
                let (icon, icon_style) = if task.is_complete {
                    ("✔", app.theme.success)
                } else if task.in_progress {
                    ("…", app.theme.warning)
                } else {
                    ("☐", app.theme.pending)
                };
                title_spans.push(Span::styled(format!("{} ", icon), icon_style));
            } else {
                // Note icon
                title_spans.push(Span::styled("● ", app.theme.info));
            }

            title_spans.push(Span::styled(item.description().to_string(), desc_style));

            lines.push(Line::from(title_spans));
            item_line_map.push(Some(item.id()));

            // Render body if present (for notes)
            if let Some(note) = item.as_note() {
                if let Some(body) = note.body() {
                    for line in body.lines() {
                        let body_style = if is_selected {
                            app.theme.selected
                        } else {
                            app.theme.muted
                        };
                        // Indent body
                        lines.push(Line::from(vec![
                            Span::raw("        "),
                            Span::styled(line.to_string(), body_style),
                        ]));
                        item_line_map.push(Some(item.id()));
                    }
                }
            }
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
    let selected_line = item_line_map
        .iter()
        .position(|id| *id == selected_id)
        .unwrap_or(0);

    let scroll_offset = if selected_line >= area.height as usize {
        selected_line.saturating_sub(area.height as usize / 2)
    } else {
        0
    };

    let paragraph = Paragraph::new(lines.clone()).scroll((scroll_offset as u16, 0));
    frame.render_widget(paragraph, area);

    if lines.len() > area.height as usize {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None);
        let mut scrollbar_state = ScrollbarState::new(lines.len()).position(scroll_offset);
        frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    }
}
