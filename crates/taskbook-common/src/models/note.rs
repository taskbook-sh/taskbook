use std::borrow::Cow;

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

    /// Note title (kept as "description" for JSON backward compatibility)
    pub description: String,

    /// Optional note body content for rich notes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    #[serde(rename = "isStarred")]
    pub is_starred: bool,

    #[serde(deserialize_with = "board::deserialize_boards")]
    pub boards: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
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
            body: None,
            is_starred: false,
            boards,
            tags: Vec::new(),
        }
    }

    /// Create a note with both title and body content
    pub fn new_with_body(
        id: u64,
        description: String,
        body: Option<String>,
        boards: Vec<String>,
    ) -> Self {
        let now = chrono::Local::now();
        Self {
            id,
            date: now.format("%a %b %d %Y").to_string(),
            timestamp: now.timestamp_millis(),
            is_task_flag: false,
            description,
            body,
            is_starred: false,
            boards,
            tags: Vec::new(),
        }
    }

    /// Create a note with tags
    pub fn new_with_tags(
        id: u64,
        description: String,
        boards: Vec<String>,
        tags: Vec<String>,
    ) -> Self {
        let mut note = Self::new(id, description, boards);
        note.tags = tags;
        note
    }

    /// Returns the note title (alias for description)
    pub fn title(&self) -> &str {
        &self.description
    }

    /// Returns the note body content, if any
    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }

    /// Returns true if the note has non-empty body content
    pub fn has_body(&self) -> bool {
        self.body.as_ref().is_some_and(|b| !b.trim().is_empty())
    }

    /// Returns the full note content (title + body combined).
    /// Returns a borrowed reference when there is no body to avoid allocation.
    pub fn full_content(&self) -> Cow<'_, str> {
        match &self.body {
            Some(body) if !body.trim().is_empty() => {
                Cow::Owned(format!("{}\n\n{}", self.description, body))
            }
            _ => Cow::Borrowed(&self.description),
        }
    }

    /// Set the note body content
    pub fn set_body(&mut self, body: Option<String>) {
        self.body = body;
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

    fn tags(&self) -> &[String] {
        &self.tags
    }

    fn is_task(&self) -> bool {
        self.is_task_flag
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_is_task_uses_flag() {
        let note = Note::new(1, "Test".to_string(), vec!["My Board".to_string()]);
        assert!(!note.is_task());
        assert!(!note.is_task_flag);
    }

    #[test]
    fn test_note_without_body() {
        let note = Note::new(1, "Test title".to_string(), vec!["My Board".to_string()]);
        assert_eq!(note.title(), "Test title");
        assert!(note.body().is_none());
        assert!(!note.has_body());
        assert_eq!(note.full_content().as_ref(), "Test title");
    }

    #[test]
    fn test_note_with_body() {
        let note = Note::new_with_body(
            1,
            "Test title".to_string(),
            Some("Body content here.".to_string()),
            vec!["My Board".to_string()],
        );
        assert_eq!(note.title(), "Test title");
        assert_eq!(note.body(), Some("Body content here."));
        assert!(note.has_body());
        assert_eq!(
            note.full_content().as_ref(),
            "Test title\n\nBody content here."
        );
    }

    #[test]
    fn test_note_with_empty_body() {
        let note = Note::new_with_body(
            1,
            "Test title".to_string(),
            Some("   ".to_string()),
            vec!["My Board".to_string()],
        );
        // Empty/whitespace-only body should be treated as no body
        assert!(!note.has_body());
        assert_eq!(note.full_content().as_ref(), "Test title");
    }

    #[test]
    fn test_note_set_body() {
        let mut note = Note::new(1, "Test title".to_string(), vec!["My Board".to_string()]);
        assert!(!note.has_body());

        note.set_body(Some("New body".to_string()));
        assert!(note.has_body());
        assert_eq!(note.body(), Some("New body"));

        note.set_body(None);
        assert!(!note.has_body());
    }

    #[test]
    fn test_note_serialization_backward_compatibility() {
        // Old notes without body should deserialize correctly
        let json = r#"{
            "_id": 1,
            "_date": "Mon Jan 01 2024",
            "_timestamp": 1704067200000,
            "_isTask": false,
            "description": "Old note",
            "isStarred": false,
            "boards": ["My Board"]
        }"#;

        let note: Note =
            serde_json::from_str(json).expect("Failed to deserialize note without body");
        assert_eq!(note.id, 1);
        assert_eq!(note.description, "Old note");
        assert!(note.body.is_none());
    }

    #[test]
    fn test_note_serialization_with_body() {
        let note = Note::new_with_body(
            1,
            "Test title".to_string(),
            Some("Body content".to_string()),
            vec!["My Board".to_string()],
        );

        let json = serde_json::to_string(&note).expect("Failed to serialize");
        assert!(json.contains("\"body\":\"Body content\""));

        let deserialized: Note = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized.body, Some("Body content".to_string()));
    }

    #[test]
    fn test_note_serialization_without_body_skips_field() {
        let note = Note::new(1, "Test title".to_string(), vec!["My Board".to_string()]);
        let json = serde_json::to_string(&note).expect("Failed to serialize");
        // body field should be omitted when None
        assert!(!json.contains("\"body\""));
    }
}
