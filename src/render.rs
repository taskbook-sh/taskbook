use std::collections::HashMap;

use colored::Colorize;

use crate::config::Config;
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
}

impl Render {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    fn color_boards(&self, boards: &[String]) -> String {
        boards
            .iter()
            .map(|b| b.dimmed().to_string())
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
            format!("{}d", age).dimmed().to_string()
        }
    }

    fn get_correlation(&self, items: &[&StorageItem]) -> String {
        let stats = self.get_item_stats(items);
        format!("[{}/{}]", stats.complete, stats.tasks)
            .dimmed()
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
            "★".yellow().to_string()
        } else {
            String::new()
        }
    }

    fn build_prefix(&self, item: &StorageItem) -> String {
        let id = item.id();
        let id_str = id.to_string();
        let padding = " ".repeat(4 - id_str.len());
        format!("{}{}", padding, format!("{}.", id).dimmed())
    }

    fn build_message(&self, item: &StorageItem) -> String {
        if let Some(task) = item.as_task() {
            let description = &task.description;
            let priority = task.priority;

            if !task.is_complete && priority > 1 {
                let msg = if priority == 2 {
                    description.yellow().underline().to_string()
                } else {
                    description.red().underline().to_string()
                };

                let indicator = if priority == 2 {
                    "(!)".yellow().to_string()
                } else {
                    "(!!)".red().to_string()
                };

                format!("{} {}", msg, indicator)
            } else if task.is_complete {
                description.dimmed().to_string()
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
            format!("{} {}", title.underline(), "[Today]".dimmed())
        } else {
            title.underline().to_string()
        };

        let correlation = self.get_correlation(items);
        println!("\n {} {} {}", display_title, correlation, "");
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
                "✔".green().to_string()
            } else if task.in_progress {
                "…".yellow().to_string()
            } else {
                "☐".magenta().to_string()
            }
        } else {
            "●".blue().to_string()
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
            format!("{}%", stats.percent).green().to_string()
        } else if stats.percent >= 50 {
            format!("{}%", stats.percent).yellow().to_string()
        } else {
            format!("{}%", stats.percent)
        };

        let status = format!(
            "{} {} {} {} {} {} {} {}",
            stats.complete.to_string().green(),
            "done".dimmed(),
            "·".dimmed(),
            stats.in_progress.to_string().blue(),
            "in-progress".dimmed(),
            "·".dimmed(),
            stats.pending.to_string().magenta(),
            "pending".dimmed(),
        );

        let notes_word = if stats.notes == 1 { "note" } else { "notes" };
        let notes_status = format!(
            "{} {} {}",
            "·".dimmed(),
            stats.notes.to_string().blue(),
            notes_word.dimmed()
        );

        if stats.pending + stats.in_progress + stats.complete + stats.notes == 0 {
            println!("\n  Type `tb --help` to get started");
        }

        println!(
            "\n  {}",
            format!("{} of all tasks complete.", percent_str).dimmed()
        );
        println!("  {} {}\n", status, notes_status);
    }

    pub fn invalid_custom_app_dir(&self, path: &str) {
        eprintln!(
            "\n {} Custom app directory was not found on your system: {}",
            "✖".red(),
            path.red()
        );
    }

    pub fn missing_taskbook_dir_flag_value(&self) {
        eprintln!(
            "\n  {} Please provide a value for --taskbook-dir or remove the flag.",
            "✖".red()
        );
    }

    pub fn invalid_id(&self, id: u64) {
        eprintln!(
            "\n {} Unable to find item with id: {}",
            "✖".red(),
            id.to_string().dimmed()
        );
    }

    pub fn invalid_ids_number(&self) {
        eprintln!("\n {} More than one ids were given as input", "✖".red());
    }

    pub fn invalid_priority(&self) {
        eprintln!("\n {} Priority can only be 1, 2 or 3", "✖".red());
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
            "✔".green(),
            word,
            ids_str.dimmed()
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
            "✔".green(),
            word,
            ids_str.dimmed()
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
            "✔".green(),
            word,
            ids_str.dimmed()
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
        println!("\n {} Paused {}: {}", "✔".green(), word, ids_str.dimmed());
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
            "✔".green(),
            word,
            ids_str.dimmed()
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
            "✔".green(),
            word,
            ids_str.dimmed()
        );
    }

    pub fn missing_boards(&self) {
        eprintln!("\n {} No boards were given as input", "✖".red());
    }

    pub fn missing_desc(&self) {
        eprintln!("\n {} No description was given as input", "✖".red());
    }

    pub fn missing_id(&self) {
        eprintln!("\n {} No id was given as input", "✖".red());
    }

    pub fn success_create(&self, id: u64, is_task: bool) {
        let item_type = if is_task { "task:" } else { "note:" };
        println!(
            "\n {} Created {} {}",
            "✔".green(),
            item_type,
            id.to_string().dimmed()
        );
    }

    pub fn success_edit(&self, id: u64) {
        println!(
            "\n {} Updated description of item: {}",
            "✔".green(),
            id.to_string().dimmed()
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
            "✔".green(),
            word,
            ids_str.dimmed()
        );
    }

    pub fn success_move(&self, id: u64, boards: &[String]) {
        let boards_str = boards.join(", ");
        println!(
            "\n {} Move item: {} to {}",
            "✔".green(),
            id.to_string().dimmed(),
            boards_str.dimmed()
        );
    }

    pub fn success_priority(&self, id: u64, level: u8) {
        let level_str = match level {
            3 => "high".red().to_string(),
            2 => "medium".yellow().to_string(),
            _ => "normal".green().to_string(),
        };
        println!(
            "\n {} Updated priority of task: {} to {}",
            "✔".green(),
            id.to_string().dimmed(),
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
            "✔".green(),
            word,
            ids_str.dimmed()
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
            "✔".green(),
            word,
            ids_str.dimmed()
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
            "✔".green(),
            ids_str.dimmed()
        );
    }
}
