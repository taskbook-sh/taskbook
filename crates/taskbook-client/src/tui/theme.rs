use ratatui::style::{Color, Modifier, Style};

use crate::config::ThemeColors;

/// TUI theme with ratatui styles
#[derive(Debug, Clone)]
pub struct TuiTheme {
    pub muted: Style,
    pub success: Style,
    pub warning: Style,
    pub error: Style,
    pub info: Style,
    pub pending: Style,
    pub starred: Style,
    pub selected: Style,
    pub border: Style,
    pub title: Style,
    pub header: Style,
    #[allow(dead_code)]
    pub item_id: Style,
    pub completed_text: Style,
    pub board_name: Style,
}

impl From<&ThemeColors> for TuiTheme {
    fn from(colors: &ThemeColors) -> Self {
        Self {
            muted: Style::default().fg(Color::Rgb(colors.muted.r, colors.muted.g, colors.muted.b)),
            success: Style::default().fg(Color::Rgb(
                colors.success.r,
                colors.success.g,
                colors.success.b,
            )),
            warning: Style::default().fg(Color::Rgb(
                colors.warning.r,
                colors.warning.g,
                colors.warning.b,
            )),
            error: Style::default().fg(Color::Rgb(colors.error.r, colors.error.g, colors.error.b)),
            info: Style::default().fg(Color::Rgb(colors.info.r, colors.info.g, colors.info.b)),
            pending: Style::default().fg(Color::Rgb(
                colors.pending.r,
                colors.pending.g,
                colors.pending.b,
            )),
            starred: Style::default().fg(Color::Rgb(
                colors.starred.r,
                colors.starred.g,
                colors.starred.b,
            )),
            selected: Style::default()
                .bg(Color::Rgb(50, 50, 70))
                .add_modifier(Modifier::BOLD),
            border: Style::default().fg(Color::Rgb(80, 80, 100)),
            title: Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
            // Board headers - use info color for better visibility
            header: Style::default()
                .fg(Color::Rgb(colors.info.r, colors.info.g, colors.info.b))
                .add_modifier(Modifier::BOLD),
            // Item IDs - brighter than muted
            item_id: Style::default().fg(Color::Rgb(180, 180, 200)),
            // Completed task text - same color as normal text with strikethrough
            completed_text: Style::default()
                .fg(Color::Rgb(140, 140, 160))
                .add_modifier(Modifier::CROSSED_OUT),
            // Board name in headers
            board_name: Style::default()
                .fg(Color::Rgb(colors.info.r, colors.info.g, colors.info.b))
                .add_modifier(Modifier::BOLD),
        }
    }
}
