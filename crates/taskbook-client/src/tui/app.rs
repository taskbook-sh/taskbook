use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::error::Result;
use crate::render::Stats;
use crate::taskbook::Taskbook;
use taskbook_common::board;
use taskbook_common::StorageItem;

use super::theme::TuiTheme;

/// Main application state
pub struct App {
    /// Core taskbook instance for business logic
    pub taskbook: Taskbook,
    /// Current view mode
    pub view: ViewMode,
    /// Currently selected item index (global flat index)
    pub selected_index: usize,
    /// List of boards for navigation
    pub boards: Vec<String>,
    /// Cached items grouped by board/date
    pub items: HashMap<String, StorageItem>,
    /// Active popup/dialog state
    pub popup: Option<PopupState>,
    /// Status message (success/error feedback)
    pub status_message: Option<StatusMessage>,
    /// Filter state
    pub filter: FilterState,
    /// Application running flag
    pub running: bool,
    /// Theme colors for rendering
    pub theme: TuiTheme,
    /// Configuration
    pub config: Config,
    /// Flat list of item IDs in display order (for navigation)
    pub display_order: Vec<u64>,
    /// Cached statistics (recalculated on refresh)
    cached_stats: Stats,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Board,
    Timeline,
    Archive,
}

#[derive(Debug, Clone)]
pub enum PopupState {
    Help,
    EditItem {
        id: u64,
        input: String,
        cursor: usize,
    },
    Search {
        input: String,
        cursor: usize,
    },
    SelectBoardForMove {
        id: u64,
        selected: usize,
    },
    SetPriority {
        id: u64,
    },
    ConfirmDelete {
        ids: Vec<u64>,
    },
    ConfirmClear,
    // Board picker for task/note creation
    SelectBoardForTask {
        selected: usize,
    },
    SelectBoardForNote {
        selected: usize,
    },
    // Create new board
    CreateBoard {
        input: String,
        cursor: usize,
    },
    // Task/note input after board selection
    CreateTaskWithBoard {
        board: String,
        input: String,
        cursor: usize,
    },
    CreateNoteWithBoard {
        board: String,
        input: String,
        cursor: usize,
    },
    // Rename board
    RenameBoard {
        old_name: String,
        input: String,
        cursor: usize,
    },
}

#[derive(Debug, Clone, Default)]
pub struct FilterState {
    #[allow(dead_code)]
    pub attributes: Vec<String>,
    pub search_term: Option<String>,
    /// Filter to show only items from this board
    pub board_filter: Option<String>,
    /// Hide completed tasks
    pub hide_completed: bool,
}

#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub text: String,
    pub kind: StatusKind,
    pub expires_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum StatusKind {
    Success,
    Error,
    Info,
}

impl App {
    pub fn new(taskbook_dir: Option<&Path>) -> Result<Self> {
        let taskbook = Taskbook::new(taskbook_dir)?;
        let config = Config::load().unwrap_or_default();
        let theme = TuiTheme::from(&config.theme.resolve());

        let mut app = Self {
            taskbook,
            view: ViewMode::Board,
            selected_index: 0,
            boards: Vec::new(),
            items: HashMap::new(),
            popup: None,
            status_message: None,
            filter: FilterState::default(),
            running: true,
            theme,
            config,
            display_order: Vec::new(),
            cached_stats: Stats {
                percent: 0,
                complete: 0,
                in_progress: 0,
                pending: 0,
                notes: 0,
            },
        };

        app.refresh_items()?;
        Ok(app)
    }

    /// Refresh items from storage
    pub fn refresh_items(&mut self) -> Result<()> {
        self.items = self.taskbook.get_all_items()?;
        self.boards = self.taskbook.get_all_boards()?;
        self.update_display_order();
        self.recalculate_stats();

        // Clamp selection to valid range
        if !self.display_order.is_empty() && self.selected_index >= self.display_order.len() {
            self.selected_index = self.display_order.len() - 1;
        }

        Ok(())
    }

    /// Recalculate cached statistics
    fn recalculate_stats(&mut self) {
        let mut complete = 0;
        let mut in_progress = 0;
        let mut pending = 0;
        let mut notes = 0;

        for item in self.items.values() {
            if let Some(task) = item.as_task() {
                if task.is_complete {
                    complete += 1;
                } else if task.in_progress {
                    in_progress += 1;
                } else {
                    pending += 1;
                }
            } else {
                notes += 1;
            }
        }

        let total = complete + pending + in_progress;
        let percent = if total == 0 {
            0
        } else {
            (complete * 100 / total) as u32
        };

        self.cached_stats = Stats {
            percent,
            complete,
            in_progress,
            pending,
            notes,
        };
    }

    /// Check if an item should be shown based on current filters
    fn should_show_item(&self, item: &StorageItem) -> bool {
        if self.filter.hide_completed {
            if let Some(task) = item.as_task() {
                if task.is_complete {
                    return false;
                }
            }
        }
        true
    }

    /// Update the flat display order of items
    pub fn update_display_order(&mut self) {
        self.display_order.clear();

        match self.view {
            ViewMode::Board => {
                // If filtering by board, only show that board
                let boards_to_show: Vec<String> =
                    if let Some(ref filter_board) = self.filter.board_filter {
                        vec![filter_board.clone()]
                    } else {
                        self.boards.clone()
                    };

                // Order by board, then by ID within each board
                for board in &boards_to_show {
                    let mut board_items: Vec<_> = self
                        .items
                        .values()
                        .filter(|item| {
                            item.boards().iter().any(|b| board::board_eq(b, board))
                                && self.should_show_item(item)
                        })
                        .collect();
                    board_items.sort_by_key(|item| item.id());
                    for item in board_items {
                        if !self.display_order.contains(&item.id()) {
                            self.display_order.push(item.id());
                        }
                    }
                }
            }
            ViewMode::Timeline | ViewMode::Archive => {
                // Order by date (newest first), then by ID
                let mut items: Vec<_> = self
                    .items
                    .values()
                    .filter(|item| self.should_show_item(item))
                    .collect();
                items.sort_by(|a, b| {
                    b.timestamp()
                        .cmp(&a.timestamp())
                        .then_with(|| a.id().cmp(&b.id()))
                });
                for item in items {
                    self.display_order.push(item.id());
                }
            }
        }
    }

    /// Toggle hide completed tasks
    pub fn toggle_hide_completed(&mut self) {
        self.filter.hide_completed = !self.filter.hide_completed;
        self.update_display_order();
        // Clamp selection
        if !self.display_order.is_empty() && self.selected_index >= self.display_order.len() {
            self.selected_index = self.display_order.len().saturating_sub(1);
        }
    }

    /// Get the board that the currently selected item belongs to
    pub fn get_board_for_selected(&self) -> Option<String> {
        self.selected_item()
            .and_then(|item| item.boards().first().cloned())
    }

    /// Set board filter
    pub fn set_board_filter(&mut self, board: Option<String>) {
        self.filter.board_filter = board;
        self.selected_index = 0;
        self.update_display_order();
    }

    /// Clear board filter
    pub fn clear_board_filter(&mut self) {
        self.filter.board_filter = None;
        self.selected_index = 0;
        self.update_display_order();
    }

    /// Get the currently selected item ID
    pub fn selected_id(&self) -> Option<u64> {
        self.display_order.get(self.selected_index).copied()
    }

    /// Get the currently selected item
    pub fn selected_item(&self) -> Option<&StorageItem> {
        self.selected_id()
            .and_then(|id| self.items.get(&id.to_string()))
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.selected_index + 1 < self.display_order.len() {
            self.selected_index += 1;
        }
    }

    /// Go to first item
    pub fn select_first(&mut self) {
        self.selected_index = 0;
    }

    /// Go to last item
    pub fn select_last(&mut self) {
        if !self.display_order.is_empty() {
            self.selected_index = self.display_order.len() - 1;
        }
    }

    /// Set status message
    pub fn set_status(&mut self, text: String, kind: StatusKind) {
        self.status_message = Some(StatusMessage {
            text,
            kind,
            expires_at: Instant::now() + Duration::from_secs(3),
        });
    }

    /// Tick - called periodically for time-based updates
    pub fn tick(&mut self) {
        // Clear expired status messages
        if let Some(ref msg) = self.status_message {
            if Instant::now() >= msg.expires_at {
                self.status_message = None;
            }
        }
    }

    /// Get stats for the current view (returns cached value)
    pub fn get_stats(&self) -> &Stats {
        &self.cached_stats
    }

    /// Switch view mode
    pub fn set_view(&mut self, view: ViewMode) -> Result<()> {
        if self.view != view {
            self.view = view;
            self.selected_index = 0;

            // Reload data for archive view
            if view == ViewMode::Archive {
                self.items = self.taskbook.get_all_archive_items()?;
            } else {
                self.items = self.taskbook.get_all_items()?;
            }

            self.update_display_order();
            self.recalculate_stats();
        }
        Ok(())
    }

    /// Quit the application
    pub fn quit(&mut self) {
        self.running = false;
    }
}
