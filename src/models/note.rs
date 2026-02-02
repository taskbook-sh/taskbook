use serde::{Deserialize, Serialize};

use super::item::Item;
use crate::board;

/// A note item (non-task)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    #[serde(rename = "_id")]
    pub id: u64,

    #[serde(rename = "_date")]
    pub date: String,

    #[serde(rename = "_timestamp")]
    pub timestamp: i64,

    #[serde(rename = "_isTask")]
    pub is_task_flag: bool,

    pub description: String,

    #[serde(rename = "isStarred")]
    pub is_starred: bool,

    #[serde(deserialize_with = "board::deserialize_boards")]
    pub boards: Vec<String>,
}

impl Note {
    pub fn new(id: u64, description: String, boards: Vec<String>) -> Self {
        let now = chrono::Local::now();
        Self {
            id,
            date: now.format("%a %b %d %Y").to_string(),
            timestamp: now.timestamp_millis(),
            is_task_flag: false,
            description,
            is_starred: false,
            boards,
        }
    }
}

impl Item for Note {
    fn id(&self) -> u64 {
        self.id
    }

    fn date(&self) -> &str {
        &self.date
    }

    fn timestamp(&self) -> i64 {
        self.timestamp
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn is_starred(&self) -> bool {
        self.is_starred
    }

    fn boards(&self) -> &[String] {
        &self.boards
    }

    fn is_task(&self) -> bool {
        false
    }
}
