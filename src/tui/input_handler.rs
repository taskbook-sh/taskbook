use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Result of handling a text input key event
pub enum InputResult {
    /// Input was handled, here's the new state
    Changed { input: String, cursor: usize },
    /// Submit was triggered (Enter pressed)
    Submit,
    /// Cancel was triggered (Esc pressed)
    Cancel,
    /// Key was not handled
    Ignored,
}

/// Handle a key event for text input fields
/// Returns the result of handling the key
pub fn handle_text_input(key: KeyEvent, input: &str, cursor: usize) -> InputResult {
    let chars: Vec<char> = input.chars().collect();
    let char_count = chars.len();

    match key.code {
        KeyCode::Esc => InputResult::Cancel,
        KeyCode::Enter => InputResult::Submit,
        KeyCode::Backspace => {
            if cursor > 0 {
                let new_input: String = chars.iter().take(cursor - 1)
                    .chain(chars.iter().skip(cursor))
                    .collect();
                InputResult::Changed {
                    input: new_input,
                    cursor: cursor - 1,
                }
            } else {
                InputResult::Changed {
                    input: input.to_string(),
                    cursor,
                }
            }
        }
        KeyCode::Delete => {
            if cursor < char_count {
                let new_input: String = chars.iter().take(cursor)
                    .chain(chars.iter().skip(cursor + 1))
                    .collect();
                InputResult::Changed {
                    input: new_input,
                    cursor,
                }
            } else {
                InputResult::Changed {
                    input: input.to_string(),
                    cursor,
                }
            }
        }
        KeyCode::Left => {
            InputResult::Changed {
                input: input.to_string(),
                cursor: cursor.saturating_sub(1),
            }
        }
        KeyCode::Right => {
            InputResult::Changed {
                input: input.to_string(),
                cursor: (cursor + 1).min(char_count),
            }
        }
        KeyCode::Home | KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            InputResult::Changed {
                input: input.to_string(),
                cursor: 0,
            }
        }
        KeyCode::End | KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            InputResult::Changed {
                input: input.to_string(),
                cursor: char_count,
            }
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Clear line before cursor
            let new_input: String = chars.iter().skip(cursor).collect();
            InputResult::Changed {
                input: new_input,
                cursor: 0,
            }
        }
        KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Clear line after cursor
            let new_input: String = chars.iter().take(cursor).collect();
            InputResult::Changed {
                input: new_input,
                cursor,
            }
        }
        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Delete word before cursor
            if cursor > 0 {
                let before_cursor: String = chars.iter().take(cursor).collect();
                let trimmed = before_cursor.trim_end();
                let last_space = trimmed.rfind(' ').map(|i| i + 1).unwrap_or(0);
                let new_input: String = chars.iter().take(last_space)
                    .chain(chars.iter().skip(cursor))
                    .collect();
                InputResult::Changed {
                    input: new_input,
                    cursor: last_space,
                }
            } else {
                InputResult::Changed {
                    input: input.to_string(),
                    cursor,
                }
            }
        }
        KeyCode::Char(c) => {
            let new_input: String = chars.iter().take(cursor)
                .chain(std::iter::once(&c))
                .chain(chars.iter().skip(cursor))
                .collect();
            InputResult::Changed {
                input: new_input,
                cursor: cursor + 1,
            }
        }
        _ => InputResult::Ignored,
    }
}
