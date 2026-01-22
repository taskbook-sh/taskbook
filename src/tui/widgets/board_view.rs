use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::models::StorageItem;
use crate::tui::app::App;

use super::item_row::{render_item_line, ItemRowOptions};

pub fn render_board_view(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    let mut item_line_map: Vec<Option<u64>> = Vec::new();
    let row_options = ItemRowOptions::for_board_view();

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
