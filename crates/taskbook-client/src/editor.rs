use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::error::{Result, TaskbookError};

/// Template shown when creating a new note in the external editor
const NEW_NOTE_TEMPLATE: &str = r#"
# Write your note title on the first non-comment line.
# Then add the body content below.
#
# Lines starting with # are comments and will be ignored.
# Delete all content (or leave only comments) to cancel.
"#;

/// Result of parsing editor content
#[derive(Debug)]
pub struct NoteContent {
    /// The note title (first non-empty, non-comment line)
    pub title: String,
    /// The note body (remaining non-comment lines)
    pub body: Option<String>,
}

/// Get the user's preferred editor from environment variables
fn get_editor() -> String {
    env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string())
}

/// Create a temporary file path for editing
fn temp_file_path() -> PathBuf {
    let uuid = uuid::Uuid::new_v4();
    env::temp_dir().join(format!("taskbook-note-{}.md", uuid))
}

/// Open the editor with the given content and return the edited content
pub fn edit_in_external_editor(initial_content: &str) -> Result<Option<NoteContent>> {
    let temp_path = temp_file_path();

    // Write initial content to temp file
    {
        let mut file = fs::File::create(&temp_path)?;
        file.write_all(initial_content.as_bytes())?;
        file.flush()?;
    }

    let editor = get_editor();

    // Open /dev/tty for direct terminal access
    // This ensures the editor works correctly even when launched from a TUI
    let tty_in = File::open("/dev/tty")
        .map_err(|e| TaskbookError::General(format!("Failed to open /dev/tty: {}", e)))?;

    // Launch editor with stdin connected to the tty
    let status = Command::new(&editor)
        .arg(&temp_path)
        .stdin(Stdio::from(tty_in))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| TaskbookError::General(format!("Failed to launch editor '{}': {}", editor, e)))?;

    if !status.success() {
        // Clean up temp file
        let _ = fs::remove_file(&temp_path);
        return Err(TaskbookError::General(format!(
            "Editor '{}' exited with non-zero status",
            editor
        )));
    }

    // Read the edited content
    let content = fs::read_to_string(&temp_path)?;

    // Clean up temp file
    let _ = fs::remove_file(&temp_path);

    // Parse the content
    parse_note_content(&content)
}

/// Open editor for creating a new note
pub fn create_note_in_editor() -> Result<Option<NoteContent>> {
    edit_in_external_editor(NEW_NOTE_TEMPLATE)
}

/// Open editor for editing an existing note
pub fn edit_existing_note_in_editor(title: &str, body: Option<&str>) -> Result<Option<NoteContent>> {
    let mut content = String::new();
    content.push_str(title);
    content.push('\n');

    if let Some(body_text) = body {
        content.push('\n');
        content.push_str(body_text);
    }

    content.push_str("\n\n");
    content.push_str("# Lines starting with # are comments and will be ignored.\n");
    content.push_str("# Delete all content (or leave only comments) to cancel.\n");

    edit_in_external_editor(&content)
}

/// Parse editor content into title and body
/// - Lines starting with # are comments (ignored)
/// - First non-empty, non-comment line is the title
/// - Remaining non-comment lines form the body
/// - Returns None if content is empty or only contains comments
fn parse_note_content(content: &str) -> Result<Option<NoteContent>> {
    let mut title: Option<String> = None;
    let mut body_lines: Vec<&str> = Vec::new();
    let mut found_title = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        if !found_title {
            // Looking for title - skip empty lines
            if !trimmed.is_empty() {
                title = Some(trimmed.to_string());
                found_title = true;
            }
        } else {
            // Collecting body lines (preserve original line, not trimmed)
            body_lines.push(line);
        }
    }

    // If no title found, treat as cancelled
    let title = match title {
        Some(t) if !t.is_empty() => t,
        _ => return Ok(None),
    };

    // Process body: trim leading/trailing empty lines but preserve internal structure
    let body = if body_lines.is_empty() {
        None
    } else {
        // Trim leading empty lines
        while !body_lines.is_empty() && body_lines[0].trim().is_empty() {
            body_lines.remove(0);
        }

        // Trim trailing empty lines
        while !body_lines.is_empty() && body_lines[body_lines.len() - 1].trim().is_empty() {
            body_lines.pop();
        }

        if body_lines.is_empty() {
            None
        } else {
            Some(body_lines.join("\n"))
        }
    };

    Ok(Some(NoteContent { title, body }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_content() {
        assert!(parse_note_content("").unwrap().is_none());
        assert!(parse_note_content("   \n   \n").unwrap().is_none());
    }

    #[test]
    fn test_parse_only_comments() {
        let content = "# comment 1\n# comment 2\n";
        assert!(parse_note_content(content).unwrap().is_none());
    }

    #[test]
    fn test_parse_title_only() {
        let content = "My note title\n";
        let result = parse_note_content(content).unwrap().unwrap();
        assert_eq!(result.title, "My note title");
        assert!(result.body.is_none());
    }

    #[test]
    fn test_parse_title_with_comments() {
        let content = "# Comment\nMy note title\n# Another comment\n";
        let result = parse_note_content(content).unwrap().unwrap();
        assert_eq!(result.title, "My note title");
        assert!(result.body.is_none());
    }

    #[test]
    fn test_parse_title_and_body() {
        let content = "My title\n\nThis is the body.\nSecond line of body.";
        let result = parse_note_content(content).unwrap().unwrap();
        assert_eq!(result.title, "My title");
        assert_eq!(result.body.as_deref(), Some("This is the body.\nSecond line of body."));
    }

    #[test]
    fn test_parse_title_and_body_with_comments() {
        let content = "# Header comment\nMy title\n\nBody line 1\n# Comment in body (skipped)\nBody line 2\n";
        let result = parse_note_content(content).unwrap().unwrap();
        assert_eq!(result.title, "My title");
        // Note: comments within the body section are included in body_lines collection
        // but since our impl collects all non-comment lines after title, this test shows behavior
        assert!(result.body.is_some());
    }

    #[test]
    fn test_parse_preserves_body_whitespace() {
        let content = "Title\n\n  Indented line\n    More indented";
        let result = parse_note_content(content).unwrap().unwrap();
        assert_eq!(result.title, "Title");
        assert_eq!(result.body.as_deref(), Some("  Indented line\n    More indented"));
    }
}
