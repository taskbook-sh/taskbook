//! Centralized board name handling.
//!
//! Board names are stored **without** the `@` prefix. The `@` is added back
//! only at the display layer via [`display_name`].

/// The default board name used when no board is specified.
pub const DEFAULT_BOARD: &str = "My Board";

/// Normalize a raw board name to its canonical stored form.
///
/// - Strips leading `@`
/// - Trims whitespace
/// - Maps the alias `myboard` (case-insensitive) to [`DEFAULT_BOARD`]
pub fn normalize_board_name(raw: &str) -> String {
    // First trim removes outer whitespace; trim_start_matches strips the '@' prefix;
    // second trim handles whitespace between '@' and the board name (e.g. "@  board").
    let trimmed = raw.trim().trim_start_matches('@').trim();
    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("myboard")
        || trimmed.eq_ignore_ascii_case("my board")
    {
        DEFAULT_BOARD.to_string()
    } else {
        trimmed.to_string()
    }
}

/// Case-insensitive board comparison.
pub fn board_eq(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

/// Format a board name for display.
///
/// The default board is shown as-is (`My Board`). All other boards get an `@` prefix.
pub fn display_name(board: &str) -> String {
    if board_eq(board, DEFAULT_BOARD) {
        DEFAULT_BOARD.to_string()
    } else {
        format!("@{}", board)
    }
}

/// Parse CLI input words into (boards, description, priority, tags).
///
/// Words starting with `@` (and longer than 1 char) are treated as board names.
/// Words starting with `+` (and longer than 1 char) are treated as tags.
/// Words matching `p:1`, `p:2`, `p:3` set priority.
/// Everything else is the description.
///
/// If no boards are found, defaults to [`DEFAULT_BOARD`].
pub fn parse_cli_input(input: &[String]) -> (Vec<String>, String, u8, Vec<String>) {
    let mut boards = Vec::new();
    let mut tags = Vec::new();
    let mut desc = Vec::new();
    let mut priority: u8 = 1;

    for word in input {
        if is_priority_opt(word) {
            if let Ok(p) = word.trim_start_matches("p:").parse::<u8>() {
                priority = p;
            }
        } else if word.starts_with('@') && word.len() > 1 {
            boards.push(normalize_board_name(word));
        } else if word.starts_with('+') && word.len() > 1 {
            let tag = normalize_tag(word);
            if !tags.iter().any(|t: &String| t.eq_ignore_ascii_case(&tag)) {
                tags.push(tag);
            }
        } else {
            desc.push(word.clone());
        }
    }

    if boards.is_empty() {
        boards.push(DEFAULT_BOARD.to_string());
    }

    // Deduplicate boards (case-insensitive)
    let mut deduped: Vec<String> = Vec::new();
    for board in boards {
        if !deduped.iter().any(|b| board_eq(b, &board)) {
            deduped.push(board);
        }
    }

    (deduped, desc.join(" "), priority, tags)
}

/// Normalize a raw tag name: strip leading `+`, trim whitespace, lowercase.
pub fn normalize_tag(raw: &str) -> String {
    raw.trim().trim_start_matches('+').trim().to_lowercase()
}

/// Format a tag for display with `+` prefix.
pub fn display_tag(tag: &str) -> String {
    format!("+{}", tag)
}

fn is_priority_opt(s: &str) -> bool {
    matches!(s, "p:1" | "p:2" | "p:3")
}

/// Deserialize a list of board names, normalizing each one.
///
/// Used as `#[serde(deserialize_with = "...")]` on the `boards` field
/// in Task and Note structs to transparently migrate old `@`-prefixed names.
pub fn deserialize_boards<'de, D>(deserializer: D) -> std::result::Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let raw: Vec<String> = Vec::deserialize(deserializer)?;
    Ok(raw.into_iter().map(|b| normalize_board_name(&b)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_strips_at_prefix() {
        assert_eq!(normalize_board_name("@coding"), "coding");
        assert_eq!(normalize_board_name("@@double"), "double");
    }

    #[test]
    fn test_normalize_trims_whitespace() {
        assert_eq!(normalize_board_name("  @coding  "), "coding");
        assert_eq!(normalize_board_name("  coding  "), "coding");
    }

    #[test]
    fn test_normalize_myboard_alias() {
        assert_eq!(normalize_board_name("myboard"), DEFAULT_BOARD);
        assert_eq!(normalize_board_name("MyBoard"), DEFAULT_BOARD);
        assert_eq!(normalize_board_name("MYBOARD"), DEFAULT_BOARD);
        assert_eq!(normalize_board_name("My Board"), DEFAULT_BOARD);
        assert_eq!(normalize_board_name("@myboard"), DEFAULT_BOARD);
    }

    #[test]
    fn test_normalize_empty_returns_default() {
        assert_eq!(normalize_board_name(""), DEFAULT_BOARD);
        assert_eq!(normalize_board_name("   "), DEFAULT_BOARD);
        assert_eq!(normalize_board_name("@"), DEFAULT_BOARD);
    }

    #[test]
    fn test_board_eq_case_insensitive() {
        assert!(board_eq("coding", "Coding"));
        assert!(board_eq("coding", "CODING"));
        assert!(board_eq("My Board", "my board"));
        assert!(!board_eq("coding", "reviews"));
    }

    #[test]
    fn test_display_name_default_board() {
        assert_eq!(display_name(DEFAULT_BOARD), "My Board");
    }

    #[test]
    fn test_display_name_other_boards() {
        assert_eq!(display_name("coding"), "@coding");
        assert_eq!(display_name("reviews"), "@reviews");
    }

    #[test]
    fn test_parse_cli_input_basic() {
        let input: Vec<String> = vec!["@coding".into(), "Fix".into(), "bug".into()];
        let (boards, desc, priority, tags) = parse_cli_input(&input);
        assert_eq!(boards, vec!["coding"]);
        assert_eq!(desc, "Fix bug");
        assert_eq!(priority, 1);
        assert!(tags.is_empty());
    }

    #[test]
    fn test_parse_cli_input_with_priority() {
        let input: Vec<String> = vec!["@coding".into(), "Fix".into(), "bug".into(), "p:3".into()];
        let (boards, desc, priority, _) = parse_cli_input(&input);
        assert_eq!(boards, vec!["coding"]);
        assert_eq!(desc, "Fix bug");
        assert_eq!(priority, 3);
    }

    #[test]
    fn test_parse_cli_input_no_board_defaults() {
        let input: Vec<String> = vec!["Simple".into(), "task".into()];
        let (boards, desc, priority, _) = parse_cli_input(&input);
        assert_eq!(boards, vec![DEFAULT_BOARD]);
        assert_eq!(desc, "Simple task");
        assert_eq!(priority, 1);
    }

    #[test]
    fn test_parse_cli_input_dedup_boards() {
        let input: Vec<String> = vec!["@coding".into(), "@Coding".into(), "task".into()];
        let (boards, desc, _, _) = parse_cli_input(&input);
        assert_eq!(boards, vec!["coding"]);
        assert_eq!(desc, "task");
    }

    #[test]
    fn test_parse_cli_input_priority_parsing() {
        for p in 1..=3u8 {
            let input: Vec<String> = vec!["task".into(), format!("p:{p}")];
            let (_, _, priority, _) = parse_cli_input(&input);
            assert_eq!(priority, p, "expected priority {p}");
        }
    }

    #[test]
    fn test_parse_cli_input_multiple_boards() {
        let input: Vec<String> = vec!["@coding".into(), "@reviews".into(), "task".into()];
        let (boards, desc, _, _) = parse_cli_input(&input);
        assert_eq!(boards, vec!["coding", "reviews"]);
        assert_eq!(desc, "task");
    }

    #[test]
    fn test_parse_cli_input_with_tags() {
        let input: Vec<String> = vec![
            "@coding".into(),
            "+urgent".into(),
            "+frontend".into(),
            "Fix".into(),
            "login".into(),
            "bug".into(),
        ];
        let (boards, desc, _, tags) = parse_cli_input(&input);
        assert_eq!(boards, vec!["coding"]);
        assert_eq!(desc, "Fix login bug");
        assert_eq!(tags, vec!["urgent", "frontend"]);
    }

    #[test]
    fn test_parse_cli_input_dedup_tags() {
        let input: Vec<String> = vec!["+urgent".into(), "+Urgent".into(), "task".into()];
        let (_, _, _, tags) = parse_cli_input(&input);
        assert_eq!(tags, vec!["urgent"]);
    }

    #[test]
    fn test_normalize_tag() {
        assert_eq!(normalize_tag("+urgent"), "urgent");
        assert_eq!(normalize_tag("+FrontEnd"), "frontend");
        assert_eq!(normalize_tag("  +spaced  "), "spaced");
    }

    #[test]
    fn test_display_tag() {
        assert_eq!(display_tag("urgent"), "+urgent");
    }
}
