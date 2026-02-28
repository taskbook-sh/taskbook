use ratatui::{
    layout::Rect,
    text::{Line, Span},
    Frame,
};

use crate::tui::app::{sort_items_by, App};
use taskbook_common::board;
use taskbook_common::StorageItem;

use super::item_row::{render_item_line, ItemRowOptions};
use super::render_scrollable_list;

pub fn render_board_view(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    let mut item_line_map: Vec<Option<u64>> = Vec::new();
    let row_options = ItemRowOptions::for_board_view();

    // Determine which boards to show (respect filter)
    let boards_to_show: Vec<&String> = if let Some(ref filter_board) = app.filter.board_filter {
        app.boards
            .iter()
            .filter(|b| board::board_eq(b, filter_board))
            .collect()
    } else {
        app.boards.iter().collect()
    };

    // Group items by board
    let mut first_group = true;
    for board in boards_to_show {
        let board_items: Vec<&StorageItem> = app
            .items
            .values()
            .filter(|item| item.boards().iter().any(|b| board::board_eq(b, board)))
            .collect();

        if board_items.is_empty() {
            continue;
        }

        // Count stats for this board (always count all tasks for stats)
        let total_tasks: usize = board_items.iter().filter(|i| i.is_task()).count();
        let complete_tasks: usize = board_items
            .iter()
            .filter_map(|i| i.as_task())
            .filter(|t| t.is_complete)
            .count();

        // Filter items for display (respecting all active filters)
        let visible_items: Vec<&StorageItem> = board_items
            .into_iter()
            .filter(|item| app.should_show_item(item))
            .collect();

        // Skip board if all visible items are hidden
        if visible_items.is_empty() {
            continue;
        }

        // Board header (blank separator between groups, not before first)
        if !first_group {
            lines.push(Line::from(""));
            item_line_map.push(None);
        }
        first_group = false;

        let stats_text = if total_tasks > 0 {
            format!(" [{}/{}]", complete_tasks, total_tasks)
        } else {
            String::new()
        };
        let display = board::display_name(board);
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(display, app.theme.board_name),
            Span::styled(stats_text, app.theme.muted),
        ]));
        item_line_map.push(None);

        // Sort items using configured method
        let mut sorted_items = visible_items;
        sort_items_by(&mut sorted_items, app.sort_method);

        for item in sorted_items {
            let is_selected = app.selected_id() == Some(item.id());
            let line = render_item_line(app, item, is_selected, &row_options);
            lines.push(line);
            item_line_map.push(Some(item.id()));
        }
    }

    render_scrollable_list(frame, area, lines, &item_line_map, app.selected_id());
}
