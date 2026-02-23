pub mod board_view;
pub mod command_line;
pub mod help_popup;
pub mod item_row;
pub mod journal_view;
pub mod status_bar;
pub mod timeline_view;

use ratatui::{
    layout::Rect,
    text::Line,
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

/// Shared scrollable list renderer used by board, timeline, and journal views.
pub(crate) fn render_scrollable_list(
    frame: &mut Frame,
    area: Rect,
    lines: Vec<Line<'static>>,
    item_line_map: &[Option<u64>],
    selected_id: Option<u64>,
) {
    // Fall back to the top of the list when the selected item is not visible
    // (e.g., filtered out or nothing selected).
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
