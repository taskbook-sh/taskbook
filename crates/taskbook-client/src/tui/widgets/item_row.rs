use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::tui::app::App;
use taskbook_common::board;
use taskbook_common::StorageItem;

/// Options for rendering an item row
pub struct ItemRowOptions {
    pub show_boards: bool,
    pub show_age: bool,
}

impl ItemRowOptions {
    pub fn for_board_view() -> Self {
        Self {
            show_boards: false,
            show_age: true,
        }
    }

    pub fn for_timeline_view() -> Self {
        Self {
            show_boards: true,
            show_age: false,
        }
    }
}

/// Render a single item as a Line with consistent styling
pub fn render_item_line(
    app: &App,
    item: &StorageItem,
    is_selected: bool,
    options: &ItemRowOptions,
) -> Line<'static> {
    let mut spans: Vec<Span> = Vec::new();

    // Selection indicator + Item ID
    if is_selected {
        spans.push(Span::styled(format!(" >{} ", item.id()), app.theme.info));
    } else {
        spans.push(Span::styled(format!("  {} ", item.id()), app.theme.item_id));
    }

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

    // Description
    let desc = item.description().to_string();
    let desc_style = if let Some(task) = item.as_task() {
        if task.is_complete {
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

    // Note body indicator
    if item.note_has_body() {
        spans.push(Span::styled(" [...]", app.theme.muted));
    }

    // Tags
    let tags = item.tags();
    if !tags.is_empty() {
        spans.push(Span::raw(" "));
        for (i, tag) in tags.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" "));
            }
            spans.push(Span::styled(format!("+{}", tag), app.theme.info));
        }
    }

    // Boards (for timeline view)
    if options.show_boards {
        let boards: Vec<String> = item
            .boards()
            .iter()
            .filter(|b| !board::board_eq(b, board::DEFAULT_BOARD))
            .map(|b| board::display_name(b))
            .collect();
        let boards_str = boards.join(" ");
        if !boards_str.is_empty() {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(boards_str, app.theme.muted));
        }
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

    // Age (for board view)
    if options.show_age {
        let age = calculate_age(item.timestamp());
        if !age.is_empty() {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(age, app.theme.muted));
        }
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
