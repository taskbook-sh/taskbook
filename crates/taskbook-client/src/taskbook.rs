use std::collections::{HashMap, HashSet};
use std::path::Path;

use arboard::Clipboard;

use crate::config::Config;
use crate::directory::resolve_taskbook_directory;
use crate::editor;
use crate::error::{Result, TaskbookError};
use crate::render::{Render, Stats};
use crate::storage::{LocalStorage, RemoteStorage, StorageBackend};
use taskbook_common::board::{self, DEFAULT_BOARD};
use taskbook_common::{Note, StorageItem, Task};

pub struct Taskbook {
    storage: Box<dyn StorageBackend>,
    render: Render,
}

impl Taskbook {
    pub fn new(taskbook_dir: Option<&Path>) -> Result<Self> {
        let config = Config::load().unwrap_or_default();

        let storage: Box<dyn StorageBackend> = if config.sync.enabled {
            Box::new(RemoteStorage::new(&config.sync.server_url)?)
        } else {
            let resolved_dir = resolve_taskbook_directory(taskbook_dir)?;
            Box::new(LocalStorage::new(&resolved_dir)?)
        };

        let render = Render::new(config);

        Ok(Self { storage, render })
    }

    fn get_data(&self) -> Result<HashMap<String, StorageItem>> {
        self.storage.get()
    }

    fn get_archive(&self) -> Result<HashMap<String, StorageItem>> {
        self.storage.get_archive()
    }

    fn save(&self, data: &HashMap<String, StorageItem>) -> Result<()> {
        self.storage.set(data)
    }

    fn save_archive(&self, data: &HashMap<String, StorageItem>) -> Result<()> {
        self.storage.set_archive(data)
    }

    fn generate_id(&self, data: &HashMap<String, StorageItem>) -> u64 {
        let max = data
            .keys()
            .filter_map(|k| k.parse::<u64>().ok())
            .max()
            .unwrap_or(0);
        max + 1
    }

    fn remove_duplicates(&self, ids: &[u64]) -> Vec<u64> {
        let mut seen = HashSet::with_capacity(ids.len());
        ids.iter().filter(|id| seen.insert(**id)).copied().collect()
    }

    fn get_ids(&self, data: &HashMap<String, StorageItem>) -> HashSet<u64> {
        data.keys().filter_map(|k| k.parse::<u64>().ok()).collect()
    }

    /// Validate IDs without rendering errors (for TUI/silent methods)
    fn validate_ids_silent(
        &self,
        input_ids: &[u64],
        existing_ids: &HashSet<u64>,
    ) -> Result<Vec<u64>> {
        if input_ids.is_empty() {
            return Err(TaskbookError::InvalidId(0));
        }

        let unique_ids = self.remove_duplicates(input_ids);

        for id in &unique_ids {
            if !existing_ids.contains(id) {
                return Err(TaskbookError::InvalidId(*id));
            }
        }

        Ok(unique_ids)
    }

    fn validate_ids(&self, input_ids: &[u64], existing_ids: &HashSet<u64>) -> Result<Vec<u64>> {
        if input_ids.is_empty() {
            self.render.missing_id();
            return Err(TaskbookError::InvalidId(0));
        }

        let unique_ids = self.remove_duplicates(input_ids);

        for id in &unique_ids {
            if !existing_ids.contains(id) {
                self.render.invalid_id(*id);
                return Err(TaskbookError::InvalidId(*id));
            }
        }

        Ok(unique_ids)
    }

    fn get_boards(&self, data: &HashMap<String, StorageItem>) -> Vec<String> {
        let mut boards = vec![DEFAULT_BOARD.to_string()];

        // Iterate items in ID order for deterministic board discovery
        let mut items: Vec<_> = data.iter().collect();
        items.sort_by_key(|(k, _)| k.parse::<u64>().unwrap_or(u64::MAX));

        for (_, item) in &items {
            for b in item.boards() {
                if !boards.iter().any(|existing| board::board_eq(existing, b)) {
                    boards.push(b.clone());
                }
            }
        }

        // Sort non-default boards alphabetically (case-insensitive), keeping default first
        if boards.len() > 1 {
            boards[1..].sort_by_key(|a| a.to_lowercase());
        }

        boards
    }

    #[allow(dead_code)]
    fn get_dates(&self, data: &HashMap<String, StorageItem>) -> Vec<String> {
        let mut seen: HashSet<String> = HashSet::new();
        let mut dates = Vec::new();

        for item in data.values() {
            let date = item.date().to_string();
            if seen.insert(date.clone()) {
                dates.push(date);
            }
        }

        dates
    }

    fn get_options(&self, input: &[String]) -> Result<(Vec<String>, String, u64, u8)> {
        if input.is_empty() {
            self.render.missing_desc();
            return Err(TaskbookError::InvalidId(0));
        }

        let data = self.get_data()?;
        let id = self.generate_id(&data);

        let (boards, description, priority) = board::parse_cli_input(input);

        Ok((boards, description, id, priority))
    }

    fn get_stats(&self, data: &HashMap<String, StorageItem>) -> Stats {
        let mut complete = 0;
        let mut in_progress = 0;
        let mut pending = 0;
        let mut notes = 0;

        for item in data.values() {
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

        Stats {
            percent,
            complete,
            in_progress,
            pending,
            notes,
        }
    }

    fn has_terms(string: &str, terms: &[String]) -> bool {
        let string_lower = string.to_lowercase();
        for term in terms {
            if string_lower.contains(&term.to_lowercase()) {
                return true;
            }
        }
        false
    }

    fn filter_task(data: &mut HashMap<String, StorageItem>) {
        data.retain(|_, item| item.is_task());
    }

    fn filter_note(data: &mut HashMap<String, StorageItem>) {
        data.retain(|_, item| !item.is_task());
    }

    fn filter_starred(data: &mut HashMap<String, StorageItem>) {
        data.retain(|_, item| item.is_starred());
    }

    fn filter_complete(data: &mut HashMap<String, StorageItem>) {
        data.retain(|_, item| item.as_task().map(|t| t.is_complete).unwrap_or(false));
    }

    fn filter_in_progress(data: &mut HashMap<String, StorageItem>) {
        data.retain(|_, item| item.as_task().map(|t| t.in_progress).unwrap_or(false));
    }

    fn filter_pending(data: &mut HashMap<String, StorageItem>) {
        data.retain(|_, item| {
            item.as_task()
                .map(|t| !t.is_complete && !t.in_progress)
                .unwrap_or(false)
        });
    }

    fn filter_by_attributes(&self, attrs: &[String], data: &mut HashMap<String, StorageItem>) {
        for attr in attrs {
            match attr.as_str() {
                "star" | "starred" => Self::filter_starred(data),
                "done" | "checked" | "complete" => Self::filter_complete(data),
                "progress" | "started" | "begun" => Self::filter_in_progress(data),
                "pending" | "unchecked" | "incomplete" => Self::filter_pending(data),
                "todo" | "task" | "tasks" => Self::filter_task(data),
                "note" | "notes" => Self::filter_note(data),
                _ => {}
            }
        }
    }

    fn group_by_board<'a>(
        &self,
        data: &'a HashMap<String, StorageItem>,
        boards: &[String],
    ) -> HashMap<String, Vec<&'a StorageItem>> {
        let mut grouped: HashMap<String, Vec<&StorageItem>> = HashMap::new();

        for item in data.values() {
            for board in boards {
                if item.boards().iter().any(|b| board::board_eq(b, board)) {
                    grouped.entry(board.clone()).or_default().push(item);
                }
            }
        }

        grouped
    }

    fn group_by_date<'a>(
        &self,
        data: &'a HashMap<String, StorageItem>,
    ) -> HashMap<String, Vec<&'a StorageItem>> {
        let mut grouped: HashMap<String, Vec<&StorageItem>> = HashMap::new();

        for item in data.values() {
            let date = item.date().to_string();
            grouped.entry(date).or_default().push(item);
        }

        grouped
    }

    fn save_item_to_archive(&self, item: StorageItem) -> Result<()> {
        let mut archive = self.get_archive()?;
        let archive_id = self.generate_id(&archive);

        let mut item = item;
        match &mut item {
            StorageItem::Task(t) => t.id = archive_id,
            StorageItem::Note(n) => n.id = archive_id,
        }

        archive.insert(archive_id.to_string(), item);
        self.save_archive(&archive)
    }

    fn save_item_to_storage(&self, item: StorageItem) -> Result<()> {
        let mut data = self.get_data()?;
        let restore_id = self.generate_id(&data);

        let mut item = item;
        match &mut item {
            StorageItem::Task(t) => t.id = restore_id,
            StorageItem::Note(n) => n.id = restore_id,
        }

        data.insert(restore_id.to_string(), item);
        self.save(&data)
    }

    // Public API methods for TUI access

    /// Get all items without rendering (for TUI)
    pub fn get_all_items(&self) -> Result<HashMap<String, StorageItem>> {
        self.get_data()
    }

    /// Get all archived items without rendering (for TUI)
    pub fn get_all_archive_items(&self) -> Result<HashMap<String, StorageItem>> {
        self.get_archive()
    }

    /// Get all boards (for TUI)
    pub fn get_all_boards(&self) -> Result<Vec<String>> {
        let data = self.get_data()?;
        Ok(self.get_boards(&data))
    }

    // Silent methods for TUI (no render output)

    /// Create a task with explicit board and description (for TUI)
    pub fn create_task_direct(
        &self,
        boards: Vec<String>,
        description: String,
        priority: u8,
    ) -> Result<u64> {
        if description.is_empty() {
            return Err(TaskbookError::InvalidId(0));
        }

        let mut data = self.get_data()?;
        let id = self.generate_id(&data);
        let task = Task::new(id, description, boards, priority);
        data.insert(id.to_string(), StorageItem::Task(task));
        self.save(&data)?;
        Ok(id)
    }

    /// Create a note with explicit board and description (for TUI)
    pub fn create_note_direct(&self, boards: Vec<String>, description: String) -> Result<u64> {
        if description.is_empty() {
            return Err(TaskbookError::InvalidId(0));
        }

        let mut data = self.get_data()?;
        let id = self.generate_id(&data);
        let note = Note::new(id, description, boards);
        data.insert(id.to_string(), StorageItem::Note(note));
        self.save(&data)?;
        Ok(id)
    }

    /// Create a note with title and body (for TUI)
    #[allow(dead_code)]
    pub fn create_note_with_body_direct(
        &self,
        boards: Vec<String>,
        title: String,
        body: Option<String>,
    ) -> Result<u64> {
        if title.is_empty() {
            return Err(TaskbookError::InvalidId(0));
        }

        let mut data = self.get_data()?;
        let id = self.generate_id(&data);
        let note = Note::new_with_body(id, title, body, boards);
        data.insert(id.to_string(), StorageItem::Note(note));
        self.save(&data)?;
        Ok(id)
    }

    /// Edit note body without CLI output (for TUI)
    pub fn edit_note_body_silent(&self, id: u64, body: Option<String>) -> Result<()> {
        let mut data = self.get_data()?;
        let existing_ids = self.get_ids(&data);
        self.validate_ids_silent(&[id], &existing_ids)?;

        if let Some(item) = data.get_mut(&id.to_string()) {
            if !item.set_note_body(body) {
                return Err(TaskbookError::General("Item is not a note".to_string()));
            }
        }

        self.save(&data)
    }

    /// Check tasks without CLI output (for TUI)
    pub fn check_tasks_silent(&self, ids: &[u64]) -> Result<()> {
        let mut data = self.get_data()?;
        let existing_ids = self.get_ids(&data);
        let validated_ids = self.validate_ids_silent(ids, &existing_ids)?;

        for id in validated_ids {
            if let Some(item) = data.get_mut(&id.to_string()) {
                if let Some(task) = item.as_task_mut() {
                    task.in_progress = false;
                    task.is_complete = !task.is_complete;
                }
            }
        }

        self.save(&data)
    }

    /// Begin tasks without CLI output (for TUI)
    pub fn begin_tasks_silent(&self, ids: &[u64]) -> Result<()> {
        let mut data = self.get_data()?;
        let existing_ids = self.get_ids(&data);
        let validated_ids = self.validate_ids_silent(ids, &existing_ids)?;

        for id in validated_ids {
            if let Some(item) = data.get_mut(&id.to_string()) {
                if let Some(task) = item.as_task_mut() {
                    task.is_complete = false;
                    task.in_progress = !task.in_progress;
                }
            }
        }

        self.save(&data)
    }

    /// Star items without CLI output (for TUI)
    pub fn star_items_silent(&self, ids: &[u64]) -> Result<()> {
        let mut data = self.get_data()?;
        let existing_ids = self.get_ids(&data);
        let validated_ids = self.validate_ids_silent(ids, &existing_ids)?;

        for id in validated_ids {
            if let Some(item) = data.get_mut(&id.to_string()) {
                let new_starred = !item.is_starred();
                item.set_starred(new_starred);
            }
        }

        self.save(&data)
    }

    /// Delete items without CLI output (for TUI)
    pub fn delete_items_silent(&self, ids: &[u64]) -> Result<()> {
        let mut data = self.get_data()?;
        let existing_ids = self.get_ids(&data);
        let validated_ids = self.validate_ids_silent(ids, &existing_ids)?;

        for id in validated_ids {
            if let Some(item) = data.remove(&id.to_string()) {
                self.save_item_to_archive(item)?;
            }
        }

        self.save(&data)
    }

    /// Restore items without CLI output (for TUI)
    pub fn restore_items_silent(&self, ids: &[u64]) -> Result<()> {
        let mut archive = self.get_archive()?;
        let archive_ids = self.get_ids(&archive);
        let validated_ids = self.validate_ids_silent(ids, &archive_ids)?;

        for id in validated_ids {
            if let Some(item) = archive.remove(&id.to_string()) {
                self.save_item_to_storage(item)?;
            }
        }

        self.save_archive(&archive)
    }

    /// Edit description without CLI output (for TUI)
    pub fn edit_description_silent(&self, id: u64, new_desc: &str) -> Result<()> {
        let mut data = self.get_data()?;
        let existing_ids = self.get_ids(&data);
        self.validate_ids_silent(&[id], &existing_ids)?;

        if let Some(item) = data.get_mut(&id.to_string()) {
            item.set_description(new_desc.to_string());
        }

        self.save(&data)
    }

    /// Move to board without CLI output (for TUI)
    pub fn move_boards_silent(&self, id: u64, boards: Vec<String>) -> Result<()> {
        let mut data = self.get_data()?;
        let existing_ids = self.get_ids(&data);
        self.validate_ids_silent(&[id], &existing_ids)?;

        let normalized: Vec<String> = boards
            .into_iter()
            .map(|b| board::normalize_board_name(&b))
            .collect();
        if let Some(item) = data.get_mut(&id.to_string()) {
            item.set_boards(normalized);
        }

        self.save(&data)
    }

    /// Update priority without CLI output (for TUI)
    pub fn update_priority_silent(&self, id: u64, priority: u8) -> Result<()> {
        let mut data = self.get_data()?;
        let existing_ids = self.get_ids(&data);
        self.validate_ids_silent(&[id], &existing_ids)?;

        if let Some(item) = data.get_mut(&id.to_string()) {
            if let Some(task) = item.as_task_mut() {
                task.priority = priority;
            }
        }

        self.save(&data)
    }

    /// Clear completed without CLI output (for TUI)
    pub fn clear_silent(&self) -> Result<usize> {
        let data = self.get_data()?;
        let mut ids_to_delete: Vec<u64> = Vec::new();

        for (id, item) in &data {
            if let Some(task) = item.as_task() {
                if task.is_complete {
                    if let Ok(id) = id.parse::<u64>() {
                        ids_to_delete.push(id);
                    }
                }
            }
        }

        if ids_to_delete.is_empty() {
            return Ok(0);
        }

        let count = ids_to_delete.len();
        let mut data = self.get_data()?;
        for id in &ids_to_delete {
            if let Some(item) = data.remove(&id.to_string()) {
                self.save_item_to_archive(item)?;
            }
        }
        self.save(&data)?;
        Ok(count)
    }

    /// Copy to clipboard without CLI output (for TUI)
    pub fn copy_to_clipboard_silent(&self, ids: &[u64]) -> Result<()> {
        let data = self.get_data()?;
        let existing_ids = self.get_ids(&data);
        let validated_ids = self.validate_ids_silent(ids, &existing_ids)?;

        let descriptions: Vec<String> = validated_ids
            .iter()
            .filter_map(|id| data.get(&id.to_string()))
            .map(|item| item.description().to_string())
            .collect();

        if descriptions.is_empty() {
            return Err(TaskbookError::NoItemsToCopy);
        }

        let mut clipboard =
            Clipboard::new().map_err(|e| TaskbookError::Clipboard(e.to_string()))?;
        clipboard
            .set_text(descriptions.join("\n"))
            .map_err(|e| TaskbookError::Clipboard(e.to_string()))?;

        Ok(())
    }

    /// Rename a board across all items (for TUI)
    pub fn rename_board_silent(&self, old_name: &str, new_name: &str) -> Result<usize> {
        let mut data = self.get_data()?;
        let mut count = 0;
        let normalized_new = board::normalize_board_name(new_name);

        for item in data.values_mut() {
            let boards = item.boards().to_vec();
            if boards.iter().any(|b| board::board_eq(b, old_name)) {
                let new_boards: Vec<String> = boards
                    .iter()
                    .map(|b| {
                        if board::board_eq(b, old_name) {
                            normalized_new.clone()
                        } else {
                            b.clone()
                        }
                    })
                    .collect();
                item.set_boards(new_boards);
                count += 1;
            }
        }

        if count > 0 {
            self.save(&data)?;
        }

        Ok(count)
    }

    // Public API methods

    pub fn create_note(&self, desc: &[String]) -> Result<()> {
        let (boards, description, id, _) = self.get_options(desc)?;

        if description.is_empty() {
            self.render.missing_desc();
            return Err(TaskbookError::InvalidId(0));
        }

        let note = Note::new(id, description, boards);
        let mut data = self.get_data()?;
        data.insert(id.to_string(), StorageItem::Note(note));
        self.save(&data)?;
        self.render.success_create(id, false);
        Ok(())
    }

    /// Create a note using external editor
    pub fn create_note_with_editor(&self) -> Result<()> {
        let content = editor::create_note_in_editor()?;

        match content {
            Some(note_content) => {
                let mut data = self.get_data()?;
                let id = self.generate_id(&data);
                let note = Note::new_with_body(
                    id,
                    note_content.title,
                    note_content.body,
                    vec![DEFAULT_BOARD.to_string()],
                );
                data.insert(id.to_string(), StorageItem::Note(note));
                self.save(&data)?;
                self.render.success_create(id, false);
                Ok(())
            }
            None => {
                self.render.note_cancelled();
                Ok(())
            }
        }
    }

    /// Edit an existing note in external editor
    pub fn edit_note_in_editor(&self, input: &[String]) -> Result<()> {
        // Parse the ID from input (format: @<id>)
        let targets: Vec<&String> = input.iter().filter(|x| x.starts_with('@')).collect();

        if targets.is_empty() {
            self.render.missing_id();
            return Err(TaskbookError::InvalidId(0));
        }

        if targets.len() > 1 {
            self.render.invalid_ids_number();
            return Err(TaskbookError::InvalidId(0));
        }

        let target = targets[0];
        let id_str = target.trim_start_matches('@');
        let id: u64 = id_str.parse().map_err(|_| TaskbookError::InvalidId(0))?;

        let data = self.get_data()?;
        let existing_ids = self.get_ids(&data);
        let validated_ids = self.validate_ids(&[id], &existing_ids)?;
        let id = validated_ids[0];

        // Get the current note
        let item = data
            .get(&id.to_string())
            .ok_or(TaskbookError::InvalidId(id))?;

        let note = item
            .as_note()
            .ok_or_else(|| TaskbookError::General("Item is not a note".to_string()))?;

        // Open editor with current content
        let content = editor::edit_existing_note_in_editor(note.title(), note.body())?;

        match content {
            Some(note_content) => {
                let mut data = self.get_data()?;
                if let Some(item) = data.get_mut(&id.to_string()) {
                    item.set_description(note_content.title);
                    item.set_note_body(note_content.body);
                }
                self.save(&data)?;
                self.render.success_edit(id);
                Ok(())
            }
            None => {
                self.render.note_cancelled();
                Ok(())
            }
        }
    }

    pub fn create_task(&self, desc: &[String]) -> Result<()> {
        let (boards, description, id, priority) = self.get_options(desc)?;

        if description.is_empty() {
            self.render.missing_desc();
            return Err(TaskbookError::InvalidId(0));
        }

        let task = Task::new(id, description, boards, priority);
        let mut data = self.get_data()?;
        data.insert(id.to_string(), StorageItem::Task(task));
        self.save(&data)?;
        self.render.success_create(id, true);
        Ok(())
    }

    pub fn copy_to_clipboard(&self, ids: &[u64]) -> Result<()> {
        let data = self.get_data()?;
        let existing_ids = self.get_ids(&data);
        let validated_ids = self.validate_ids(ids, &existing_ids)?;

        let mut descriptions = Vec::new();
        for id in &validated_ids {
            if let Some(item) = data.get(&id.to_string()) {
                descriptions.push(item.description().to_string());
            }
        }

        if descriptions.is_empty() {
            return Err(TaskbookError::NoItemsToCopy);
        }

        let mut clipboard =
            Clipboard::new().map_err(|e| TaskbookError::Clipboard(e.to_string()))?;
        clipboard
            .set_text(descriptions.join("\n"))
            .map_err(|e| TaskbookError::Clipboard(e.to_string()))?;

        self.render.success_copy_to_clipboard(&validated_ids);
        Ok(())
    }

    pub fn check_tasks(&self, ids: &[u64]) -> Result<()> {
        let mut data = self.get_data()?;
        let existing_ids = self.get_ids(&data);
        let validated_ids = self.validate_ids(ids, &existing_ids)?;

        let mut checked = Vec::new();
        let mut unchecked = Vec::new();

        for id in &validated_ids {
            if let Some(item) = data.get_mut(&id.to_string()) {
                if let Some(task) = item.as_task_mut() {
                    task.in_progress = false;
                    task.is_complete = !task.is_complete;
                    if task.is_complete {
                        checked.push(*id);
                    } else {
                        unchecked.push(*id);
                    }
                }
            }
        }

        self.save(&data)?;
        self.render.mark_complete(&checked);
        self.render.mark_incomplete(&unchecked);
        Ok(())
    }

    pub fn begin_tasks(&self, ids: &[u64]) -> Result<()> {
        let mut data = self.get_data()?;
        let existing_ids = self.get_ids(&data);
        let validated_ids = self.validate_ids(ids, &existing_ids)?;

        let mut started = Vec::new();
        let mut paused = Vec::new();

        for id in &validated_ids {
            if let Some(item) = data.get_mut(&id.to_string()) {
                if let Some(task) = item.as_task_mut() {
                    task.is_complete = false;
                    task.in_progress = !task.in_progress;
                    if task.in_progress {
                        started.push(*id);
                    } else {
                        paused.push(*id);
                    }
                }
            }
        }

        self.save(&data)?;
        self.render.mark_started(&started);
        self.render.mark_paused(&paused);
        Ok(())
    }

    pub fn delete_items(&self, ids: &[u64]) -> Result<()> {
        let mut data = self.get_data()?;
        let existing_ids = self.get_ids(&data);
        let validated_ids = self.validate_ids(ids, &existing_ids)?;

        for id in &validated_ids {
            if let Some(item) = data.remove(&id.to_string()) {
                self.save_item_to_archive(item)?;
            }
        }

        self.save(&data)?;
        self.render.success_delete(&validated_ids);
        Ok(())
    }

    pub fn display_archive(&self) -> Result<()> {
        let archive = self.get_archive()?;
        let grouped = self.group_by_date(&archive);
        self.render.display_by_date(&grouped);
        Ok(())
    }

    pub fn display_by_board(&self) -> Result<()> {
        let data = self.get_data()?;
        let boards = self.get_boards(&data);
        let grouped = self.group_by_board(&data, &boards);
        self.render.display_by_board(&grouped);
        Ok(())
    }

    pub fn display_by_date(&self) -> Result<()> {
        let data = self.get_data()?;
        let grouped = self.group_by_date(&data);
        self.render.display_by_date(&grouped);
        Ok(())
    }

    pub fn display_stats(&self) -> Result<()> {
        let data = self.get_data()?;
        let stats = self.get_stats(&data);
        self.render.display_stats(&stats);
        Ok(())
    }

    pub fn edit_description(&self, input: &[String]) -> Result<()> {
        let targets: Vec<&String> = input.iter().filter(|x| x.starts_with('@')).collect();

        if targets.is_empty() {
            self.render.missing_id();
            return Err(TaskbookError::InvalidId(0));
        }

        if targets.len() > 1 {
            self.render.invalid_ids_number();
            return Err(TaskbookError::InvalidId(0));
        }

        let target = targets[0];
        let id_str = target.trim_start_matches('@');
        let id: u64 = id_str.parse().map_err(|_| TaskbookError::InvalidId(0))?;

        let mut data = self.get_data()?;
        let existing_ids = self.get_ids(&data);
        let validated_ids = self.validate_ids(&[id], &existing_ids)?;
        let id = validated_ids[0];

        let new_desc: String = input
            .iter()
            .filter(|x| *x != target)
            .cloned()
            .collect::<Vec<_>>()
            .join(" ");

        if new_desc.is_empty() {
            self.render.missing_desc();
            return Err(TaskbookError::InvalidId(0));
        }

        if let Some(item) = data.get_mut(&id.to_string()) {
            item.set_description(new_desc);
        }

        self.save(&data)?;
        self.render.success_edit(id);
        Ok(())
    }

    pub fn find_items(&self, terms: &[String]) -> Result<()> {
        let data = self.get_data()?;
        let mut result: HashMap<String, StorageItem> = HashMap::new();

        for (id, item) in &data {
            if Self::has_terms(item.description(), terms) {
                result.insert(id.clone(), item.clone());
            }
        }

        let boards = self.get_boards(&result);
        let grouped = self.group_by_board(&result, &boards);
        self.render.display_by_board(&grouped);
        Ok(())
    }

    pub fn list_by_attributes(&self, terms: &[String]) -> Result<()> {
        let data = self.get_data()?;
        let stored_boards = self.get_boards(&data);

        let mut boards: Vec<String> = Vec::new();
        let mut attributes: Vec<String> = Vec::new();

        for term in terms {
            let normalized = board::normalize_board_name(term);
            if stored_boards
                .iter()
                .any(|b| board::board_eq(b, &normalized))
            {
                if !boards.iter().any(|b| board::board_eq(b, &normalized)) {
                    boards.push(normalized);
                }
            } else {
                attributes.push(term.clone());
            }
        }

        let mut filtered_data = data.clone();
        self.filter_by_attributes(&attributes, &mut filtered_data);

        let display_boards = if boards.is_empty() {
            self.get_boards(&filtered_data)
        } else {
            boards
        };

        let grouped = self.group_by_board(&filtered_data, &display_boards);
        self.render.display_by_board(&grouped);
        Ok(())
    }

    pub fn move_boards(&self, input: &[String]) -> Result<()> {
        let targets: Vec<&String> = input.iter().filter(|x| x.starts_with('@')).collect();

        if targets.is_empty() {
            self.render.missing_id();
            return Err(TaskbookError::InvalidId(0));
        }

        if targets.len() > 1 {
            self.render.invalid_ids_number();
            return Err(TaskbookError::InvalidId(0));
        }

        let target = targets[0];
        let id_str = target.trim_start_matches('@');
        let id: u64 = id_str.parse().map_err(|_| TaskbookError::InvalidId(0))?;

        let mut data = self.get_data()?;
        let existing_ids = self.get_ids(&data);
        let validated_ids = self.validate_ids(&[id], &existing_ids)?;
        let id = validated_ids[0];

        let mut boards: Vec<String> = Vec::new();
        for word in input {
            if word != target {
                let normalized = board::normalize_board_name(word);
                if !boards.iter().any(|b| board::board_eq(b, &normalized)) {
                    boards.push(normalized);
                }
            }
        }

        if boards.is_empty() {
            self.render.missing_boards();
            return Err(TaskbookError::InvalidId(0));
        }

        if let Some(item) = data.get_mut(&id.to_string()) {
            item.set_boards(boards.clone());
        }

        self.save(&data)?;
        let display_boards: Vec<String> = boards.iter().map(|b| board::display_name(b)).collect();
        self.render.success_move(id, &display_boards);
        Ok(())
    }

    pub fn restore_items(&self, ids: &[u64]) -> Result<()> {
        let mut archive = self.get_archive()?;
        let archive_ids = self.get_ids(&archive);
        let validated_ids = self.validate_ids(ids, &archive_ids)?;

        for id in &validated_ids {
            if let Some(item) = archive.remove(&id.to_string()) {
                self.save_item_to_storage(item)?;
            }
        }

        self.save_archive(&archive)?;
        self.render.success_restore(&validated_ids);
        Ok(())
    }

    pub fn star_items(&self, ids: &[u64]) -> Result<()> {
        let mut data = self.get_data()?;
        let existing_ids = self.get_ids(&data);
        let validated_ids = self.validate_ids(ids, &existing_ids)?;

        let mut starred = Vec::new();
        let mut unstarred = Vec::new();

        for id in &validated_ids {
            if let Some(item) = data.get_mut(&id.to_string()) {
                let new_starred = !item.is_starred();
                item.set_starred(new_starred);
                if new_starred {
                    starred.push(*id);
                } else {
                    unstarred.push(*id);
                }
            }
        }

        self.save(&data)?;
        self.render.mark_starred(&starred);
        self.render.mark_unstarred(&unstarred);
        Ok(())
    }

    pub fn update_priority(&self, input: &[String]) -> Result<()> {
        let level = input
            .iter()
            .find(|x| matches!(x.as_str(), "1" | "2" | "3"))
            .map(|s| s.parse::<u8>().unwrap());

        let level = match level {
            Some(l) => l,
            None => {
                self.render.invalid_priority();
                return Err(TaskbookError::InvalidId(0));
            }
        };

        let targets: Vec<&String> = input.iter().filter(|x| x.starts_with('@')).collect();

        if targets.is_empty() {
            self.render.missing_id();
            return Err(TaskbookError::InvalidId(0));
        }

        if targets.len() > 1 {
            self.render.invalid_ids_number();
            return Err(TaskbookError::InvalidId(0));
        }

        let target = targets[0];
        let id_str = target.trim_start_matches('@');
        let id: u64 = id_str.parse().map_err(|_| TaskbookError::InvalidId(0))?;

        let mut data = self.get_data()?;
        let existing_ids = self.get_ids(&data);
        let validated_ids = self.validate_ids(&[id], &existing_ids)?;
        let id = validated_ids[0];

        if let Some(item) = data.get_mut(&id.to_string()) {
            if let Some(task) = item.as_task_mut() {
                task.priority = level;
            }
        }

        self.save(&data)?;
        self.render.success_priority(id, level);
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        let data = self.get_data()?;
        let mut ids_to_delete: Vec<u64> = Vec::new();

        for (id, item) in &data {
            if let Some(task) = item.as_task() {
                if task.is_complete {
                    if let Ok(id) = id.parse::<u64>() {
                        ids_to_delete.push(id);
                    }
                }
            }
        }

        if ids_to_delete.is_empty() {
            return Ok(());
        }

        // Delete items without the success message (we'll use success_clear instead)
        let mut data = self.get_data()?;
        for id in &ids_to_delete {
            if let Some(item) = data.remove(&id.to_string()) {
                self.save_item_to_archive(item)?;
            }
        }
        self.save(&data)?;
        self.render.success_clear(&ids_to_delete);
        Ok(())
    }
}
