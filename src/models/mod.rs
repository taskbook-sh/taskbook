mod item;
mod note;
mod task;

pub use item::Item;
pub use note::Note;
pub use task::Task;

use serde::{Deserialize, Serialize};

/// Unified storage item that can be either a Task or Note
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StorageItem {
    Task(Task),
    Note(Note),
}

impl StorageItem {
    pub fn id(&self) -> u64 {
        match self {
            StorageItem::Task(t) => t.id,
            StorageItem::Note(n) => n.id,
        }
    }

    pub fn date(&self) -> &str {
        match self {
            StorageItem::Task(t) => &t.date,
            StorageItem::Note(n) => &n.date,
        }
    }

    pub fn timestamp(&self) -> i64 {
        match self {
            StorageItem::Task(t) => t.timestamp,
            StorageItem::Note(n) => n.timestamp,
        }
    }

    pub fn description(&self) -> &str {
        match self {
            StorageItem::Task(t) => &t.description,
            StorageItem::Note(n) => &n.description,
        }
    }

    pub fn set_description(&mut self, desc: String) {
        match self {
            StorageItem::Task(t) => t.description = desc,
            StorageItem::Note(n) => n.description = desc,
        }
    }

    pub fn is_starred(&self) -> bool {
        match self {
            StorageItem::Task(t) => t.is_starred,
            StorageItem::Note(n) => n.is_starred,
        }
    }

    pub fn set_starred(&mut self, starred: bool) {
        match self {
            StorageItem::Task(t) => t.is_starred = starred,
            StorageItem::Note(n) => n.is_starred = starred,
        }
    }

    pub fn boards(&self) -> &[String] {
        match self {
            StorageItem::Task(t) => &t.boards,
            StorageItem::Note(n) => &n.boards,
        }
    }

    pub fn set_boards(&mut self, boards: Vec<String>) {
        match self {
            StorageItem::Task(t) => t.boards = boards,
            StorageItem::Note(n) => n.boards = boards,
        }
    }

    pub fn is_task(&self) -> bool {
        matches!(self, StorageItem::Task(_))
    }

    pub fn as_task(&self) -> Option<&Task> {
        match self {
            StorageItem::Task(t) => Some(t),
            StorageItem::Note(_) => None,
        }
    }

    pub fn as_task_mut(&mut self) -> Option<&mut Task> {
        match self {
            StorageItem::Task(t) => Some(t),
            StorageItem::Note(_) => None,
        }
    }

    #[allow(dead_code)]
    pub fn as_note(&self) -> Option<&Note> {
        match self {
            StorageItem::Task(_) => None,
            StorageItem::Note(n) => Some(n),
        }
    }
}
