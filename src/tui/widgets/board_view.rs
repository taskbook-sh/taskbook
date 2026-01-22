use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::models::StorageItem;
use crate::tui::app::App;

pub fn render_board_view(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    let mut item_line_map: Vec<Option<u64>> = Vec::new(); // Maps line index to item ID

    // Determine which boards to show (respect filter)
    let boards_to_show: Vec<&String> = if let Some(ref filter_board) = app.filter.board_filter {
        app.boards.iter().filter(|b| *b == filter_board).collect()
    } else {
        app.boards.iter().collect()
    };

    // Group items by board
    for board in boards_to_show {
        let board_items: Vec<&StorageItem> = app.items.values()
            .filter(|item| item.boards().contains(board))
            .collect();

        if board_items.is_empty() {
            continue;
        }

        // Count stats for this board
        let total_tasks: usize = board_items.iter().filter(|i| i.is_task()).count();
        let complete_tasks: usize = board_items.iter()
            .filter_map(|i| i.as_task())
            .filter(|t| t.is_complete)
            .count();

        // Board header
        lines.push(Line::from(""));
        item_line_map.push(None);

        // Board header with better formatting
        let stats_text = if total_tasks > 0 {
            format!(" [{}/{}]", complete_tasks, total_tasks)
        } else {
            String::new()
        };
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(board.clone(), app.theme.board_name),
            Span::styled(stats_text, app.theme.muted),
        ]));
        item_line_map.push(None);

        // Sort items by ID
        let mut sorted_items: Vec<&StorageItem> = board_items;
        sorted_items.sort_by_key(|item| item.id());

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

    // Age (days since creation)
    let age = calculate_age(item.timestamp());
    if !age.is_empty() {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(age, app.theme.muted));
    }

    let mut line = Line::from(spans);
    if is_selected {
        line = line.style(app.theme.selected);
    }
    line
}

fn calculate_age(timestamp: i64) -> String {
    let now = chrono::Utc::now().timestamp_millis();
    let diff = now - timestamp;
    let days = diff / (1000 * 60 * 60 * 24);

    if days == 0 {
        String::new()
    } else if days == 1 {
        "1d".to_string()
    } else if days < 30 {
        format!("{}d", days)
    } else if days < 365 {
        format!("{}mo", days / 30)
    } else {
        format!("{}y", days / 365)
    }
}
