use std::collections::HashMap;

use colored::{ColoredString, Colorize};

use crate::config::{Config, Rgb, ThemeColors};
use crate::models::StorageItem;

/// Statistics about items
pub struct Stats {
    pub percent: u32,
    pub complete: usize,
    pub in_progress: usize,
    pub pending: usize,
    pub notes: usize,
}

/// Item statistics for a group
struct ItemStats {
    tasks: usize,
    complete: usize,
    notes: usize,
}

pub struct Render {
    config: Config,
    theme: ThemeColors,
}

/// Trait extension for applying RGB colors
trait RgbColorize {
    fn rgb(self, color: Rgb) -> ColoredString;
}

impl<S: AsRef<str>> RgbColorize for S {
    fn rgb(self, color: Rgb) -> ColoredString {
        self.as_ref().truecolor(color.r, color.g, color.b)
    }
}

impl Render {
    pub fn new(config: Config) -> Self {
        let theme = config.theme.resolve();
        Self { config, theme }
    }

    /// Apply muted color to text
    fn muted(&self, text: &str) -> ColoredString {
        text.rgb(self.theme.muted)
    }

    /// Apply success color to text
    fn success(&self, text: &str) -> ColoredString {
        text.rgb(self.theme.success)
    }

    /// Apply warning color to text
    fn warning(&self, text: &str) -> ColoredString {
        text.rgb(self.theme.warning)
    }

    /// Apply error color to text
    fn error(&self, text: &str) -> ColoredString {
        text.rgb(self.theme.error)
    }

    /// Apply info color to text
    fn info(&self, text: &str) -> ColoredString {
        text.rgb(self.theme.info)
    }

    /// Apply pending color to text
    fn pending(&self, text: &str) -> ColoredString {
        text.rgb(self.theme.pending)
    }

    /// Apply starred color to text
    fn starred(&self, text: &str) -> ColoredString {
        text.rgb(self.theme.starred)
    }

    fn color_boards(&self, boards: &[String]) -> String {
        boards
            .iter()
            .map(|b| self.muted(b).to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn is_board_complete(&self, items: &[&StorageItem]) -> bool {
        let stats = self.get_item_stats(items);
        stats.tasks == stats.complete && stats.notes == 0
    }

    fn get_age(&self, timestamp: i64) -> String {
        let now = chrono::Utc::now().timestamp_millis();
        let daytime = 24 * 60 * 60 * 1000;
        let age = ((now - timestamp).abs() / daytime) as u32;
        if age == 0 {
            String::new()
        } else {
            self.muted(&format!("{}d", age)).to_string()
        }
    }

    fn get_correlation(&self, items: &[&StorageItem]) -> String {
        let stats = self.get_item_stats(items);
        self.muted(&format!("[{}/{}]", stats.complete, stats.tasks))
            .to_string()
    }

    fn get_item_stats(&self, items: &[&StorageItem]) -> ItemStats {
        let mut tasks = 0;
        let mut complete = 0;
        let mut notes = 0;

        for item in items {
            if item.is_task() {
                tasks += 1;
                if let Some(task) = item.as_task() {
                    if task.is_complete {
                        complete += 1;
                    }
                }
            } else {
                notes += 1;
            }
        }

        ItemStats {
            tasks,
            complete,
            notes,
        }
    }

    fn get_star(&self, item: &StorageItem) -> String {
        if item.is_starred() {
            self.starred("★").to_string()
        } else {
            String::new()
        }
    }

    fn build_prefix(&self, item: &StorageItem) -> String {
        let id = item.id();
        let id_str = id.to_string();
        let padding = " ".repeat(4 - id_str.len());
        format!("{}{}", padding, self.muted(&format!("{}.", id)))
    }

    fn build_message(&self, item: &StorageItem) -> String {
        if let Some(task) = item.as_task() {
            let description = &task.description;
            let priority = task.priority;

            if !task.is_complete && priority > 1 {
                let msg = if priority == 2 {
                    self.warning(description).underline().to_string()
                } else {
                    self.error(description).underline().to_string()
                };

                let indicator = if priority == 2 {
                    self.warning("(!)").to_string()
                } else {
                    self.error("(!!)").to_string()
                };

                format!("{} {}", msg, indicator)
            } else if task.is_complete {
                self.muted(description).strikethrough().to_string()
            } else {
                description.to_string()
            }
        } else {
            item.description().to_string()
        }
    }

    fn display_title(&self, title: &str, items: &[&StorageItem]) {
        let today = chrono::Local::now().format("%a %b %d %Y").to_string();
        let display_title = if title == today {
            format!("{} {}", title.underline(), self.muted("[Today]"))
        } else {
            title.underline().to_string()
        };

        let correlation = self.get_correlation(items);
        println!("\n {} {}", display_title, correlation);
    }

    fn display_item_by_board(&self, item: &StorageItem) {
        let age = self.get_age(item.timestamp());
        let star = self.get_star(item);
        let prefix = self.build_prefix(item);
        let message = self.build_message(item);

        let suffix = if age.is_empty() {
            star
        } else {
            format!("{} {}", age, star)
        };

        let icon = self.get_item_icon(item);
        println!("{} {} {} {}", prefix, icon, message, suffix);
    }

    fn display_item_by_date(&self, item: &StorageItem) {
        let boards: Vec<String> = item
            .boards()
            .iter()
            .filter(|b| *b != "My Board")
            .cloned()
            .collect();
        let star = self.get_star(item);
        let prefix = self.build_prefix(item);
        let message = self.build_message(item);
        let boards_str = self.color_boards(&boards);
        let suffix = format!("{} {}", boards_str, star);

        let icon = self.get_item_icon(item);
        println!("{} {} {} {}", prefix, icon, message, suffix);
    }

    fn get_item_icon(&self, item: &StorageItem) -> String {
        if let Some(task) = item.as_task() {
            if task.is_complete {
                self.success("✔").to_string()
            } else if task.in_progress {
                self.warning("…").to_string()
            } else {
                self.pending("☐").to_string()
            }
        } else {
            self.info("●").to_string()
        }
    }

    pub fn display_by_board(&self, data: &HashMap<String, Vec<&StorageItem>>) {
        let mut boards: Vec<_> = data.keys().collect();
        boards.sort();

        for board in boards {
            let items = &data[board];

            if self.is_board_complete(items) && !self.config.display_complete_tasks {
                continue;
            }

            self.display_title(board, items);

            for item in items {
                if item.is_task() {
                    if let Some(task) = item.as_task() {
                        if task.is_complete && !self.config.display_complete_tasks {
                            continue;
                        }
                    }
                }
                self.display_item_by_board(item);
            }
        }
    }

    pub fn display_by_date(&self, data: &HashMap<String, Vec<&StorageItem>>) {
        // Sort dates chronologically (most recent first based on actual date parsing)
        let mut dates: Vec<_> = data.keys().collect();
        dates.sort_by(|a, b| b.cmp(a));

        for date in dates {
            let items = &data[date];

            if self.is_board_complete(items) && !self.config.display_complete_tasks {
                continue;
            }

            self.display_title(date, items);

            for item in items {
                if item.is_task() {
                    if let Some(task) = item.as_task() {
                        if task.is_complete && !self.config.display_complete_tasks {
                            continue;
                        }
                    }
                }
                self.display_item_by_date(item);
            }
        }
    }

    pub fn display_stats(&self, stats: &Stats) {
        if !self.config.display_progress_overview {
            return;
        }

        let percent_str = if stats.percent >= 75 {
            self.success(&format!("{}%", stats.percent)).to_string()
        } else if stats.percent >= 50 {
            self.warning(&format!("{}%", stats.percent)).to_string()
        } else {
            format!("{}%", stats.percent)
        };

        let status = format!(
            "{} {} {} {} {} {} {} {}",
            self.success(&stats.complete.to_string()),
            self.muted("done"),
            self.muted("·"),
            self.info(&stats.in_progress.to_string()),
            self.muted("in-progress"),
            self.muted("·"),
            self.pending(&stats.pending.to_string()),
            self.muted("pending"),
        );

        let notes_word = if stats.notes == 1 { "note" } else { "notes" };
        let notes_status = format!(
            "{} {} {}",
            self.muted("·"),
            self.info(&stats.notes.to_string()),
            self.muted(notes_word)
        );

        if stats.pending + stats.in_progress + stats.complete + stats.notes == 0 {
            println!("\n  Type `tb --help` to get started");
        }

        println!(
            "\n  {}",
            self.muted(&format!("{} of all tasks complete.", percent_str))
        );
        println!("  {} {}\n", status, notes_status);
    }

    #[allow(dead_code)]
    pub fn invalid_custom_app_dir(&self, path: &str) {
        eprintln!(
            "\n {} Custom app directory was not found on your system: {}",
            self.error("✖"),
            self.error(path)
        );
    }

    #[allow(dead_code)]
    pub fn missing_taskbook_dir_flag_value(&self) {
        eprintln!(
            "\n  {} Please provide a value for --taskbook-dir or remove the flag.",
            self.error("✖")
        );
    }

    pub fn invalid_id(&self, id: u64) {
        eprintln!(
            "\n {} Unable to find item with id: {}",
            self.error("✖"),
            self.muted(&id.to_string())
        );
    }

    pub fn invalid_ids_number(&self) {
        eprintln!("\n {} More than one ids were given as input", self.error("✖"));
    }

    pub fn invalid_priority(&self) {
        eprintln!("\n {} Priority can only be 1, 2 or 3", self.error("✖"));
    }

    pub fn mark_complete(&self, ids: &[u64]) {
        if ids.is_empty() {
            return;
        }
        let ids_str = ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let word = if ids.len() > 1 { "tasks" } else { "task" };
        println!(
            "\n {} Checked {}: {}",
            self.success("✔"),
            word,
            self.muted(&ids_str)
        );
    }

    pub fn mark_incomplete(&self, ids: &[u64]) {
        if ids.is_empty() {
            return;
        }
        let ids_str = ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let word = if ids.len() > 1 { "tasks" } else { "task" };
        println!(
            "\n {} Unchecked {}: {}",
            self.success("✔"),
            word,
            self.muted(&ids_str)
        );
    }

    pub fn mark_started(&self, ids: &[u64]) {
        if ids.is_empty() {
            return;
        }
        let ids_str = ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let word = if ids.len() > 1 { "tasks" } else { "task" };
        println!(
            "\n {} Started {}: {}",
            self.success("✔"),
            word,
            self.muted(&ids_str)
        );
    }

    pub fn mark_paused(&self, ids: &[u64]) {
        if ids.is_empty() {
            return;
        }
        let ids_str = ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let word = if ids.len() > 1 { "tasks" } else { "task" };
        println!("\n {} Paused {}: {}", self.success("✔"), word, self.muted(&ids_str));
    }

    pub fn mark_starred(&self, ids: &[u64]) {
        if ids.is_empty() {
            return;
        }
        let ids_str = ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let word = if ids.len() > 1 { "items" } else { "item" };
        println!(
            "\n {} Starred {}: {}",
            self.success("✔"),
            word,
            self.muted(&ids_str)
        );
    }

    pub fn mark_unstarred(&self, ids: &[u64]) {
        if ids.is_empty() {
            return;
        }
        let ids_str = ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let word = if ids.len() > 1 { "items" } else { "item" };
        println!(
            "\n {} Unstarred {}: {}",
            self.success("✔"),
            word,
            self.muted(&ids_str)
        );
    }

    pub fn missing_boards(&self) {
        eprintln!("\n {} No boards were given as input", self.error("✖"));
    }

    pub fn missing_desc(&self) {
        eprintln!("\n {} No description was given as input", self.error("✖"));
    }

    pub fn missing_id(&self) {
        eprintln!("\n {} No id was given as input", self.error("✖"));
    }

    pub fn success_create(&self, id: u64, is_task: bool) {
        let item_type = if is_task { "task:" } else { "note:" };
        println!(
            "\n {} Created {} {}",
            self.success("✔"),
            item_type,
            self.muted(&id.to_string())
        );
    }

    pub fn success_edit(&self, id: u64) {
        println!(
            "\n {} Updated description of item: {}",
            self.success("✔"),
            self.muted(&id.to_string())
        );
    }

    pub fn success_delete(&self, ids: &[u64]) {
        let ids_str = ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let word = if ids.len() > 1 { "items" } else { "item" };
        println!(
            "\n {} Deleted {}: {}",
            self.success("✔"),
            word,
            self.muted(&ids_str)
        );
    }

    pub fn success_move(&self, id: u64, boards: &[String]) {
        let boards_str = boards.join(", ");
        println!(
            "\n {} Move item: {} to {}",
            self.success("✔"),
            self.muted(&id.to_string()),
            self.muted(&boards_str)
        );
    }

    pub fn success_priority(&self, id: u64, level: u8) {
        let level_str = match level {
            3 => self.error("high").to_string(),
            2 => self.warning("medium").to_string(),
            _ => self.success("normal").to_string(),
        };
        println!(
            "\n {} Updated priority of task: {} to {}",
            self.success("✔"),
            self.muted(&id.to_string()),
            level_str
        );
    }

    pub fn success_restore(&self, ids: &[u64]) {
        let ids_str = ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let word = if ids.len() > 1 { "items" } else { "item" };
        println!(
            "\n {} Restored {}: {}",
            self.success("✔"),
            word,
            self.muted(&ids_str)
        );
    }

    pub fn success_copy_to_clipboard(&self, ids: &[u64]) {
        let ids_str = ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let word = if ids.len() > 1 {
            "descriptions of items"
        } else {
            "description of item"
        };
        println!(
            "\n {} Copied the {}: {}",
            self.success("✔"),
            word,
            self.muted(&ids_str)
        );
    }

    pub fn success_clear(&self, ids: &[u64]) {
        if ids.is_empty() {
            return;
        }
        let ids_str = ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "\n {} Deleted all checked items: {}",
            self.success("✔"),
            self.muted(&ids_str)
        );
    }
}
