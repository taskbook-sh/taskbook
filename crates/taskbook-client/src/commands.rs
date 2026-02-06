use std::path::PathBuf;

use base64::Engine;
use colored::Colorize;

use crate::api_client::{ApiClient, EncryptedItemData};
use crate::config::Config;
use crate::credentials::Credentials;
use crate::directory::resolve_taskbook_directory;
use crate::error::{Result, TaskbookError};
use crate::storage::{LocalStorage, StorageBackend};
use crate::taskbook::Taskbook;
use taskbook_common::encryption::encrypt_item;

/// Execute CLI commands
#[allow(clippy::too_many_arguments)]
pub fn run(
    input: Vec<String>,
    archive: bool,
    task: bool,
    restore: bool,
    note: bool,
    delete: bool,
    check: bool,
    begin: bool,
    star: bool,
    priority: bool,
    copy: bool,
    timeline: bool,
    find: bool,
    list: bool,
    edit: bool,
    edit_note: bool,
    r#move: bool,
    clear: bool,
    taskbook_dir: Option<PathBuf>,
) -> Result<()> {
    let taskbook = Taskbook::new(taskbook_dir.as_deref())?;

    if archive {
        return taskbook.display_archive();
    }

    if task {
        return taskbook.create_task(&input);
    }

    if restore {
        let ids: Vec<u64> = input.iter().filter_map(|s| s.parse().ok()).collect();
        return taskbook.restore_items(&ids);
    }

    if note {
        // If no description provided, open external editor
        if input.is_empty() {
            return taskbook.create_note_with_editor();
        }
        return taskbook.create_note(&input);
    }

    if edit_note {
        return taskbook.edit_note_in_editor(&input);
    }

    if delete {
        let ids: Vec<u64> = input.iter().filter_map(|s| s.parse().ok()).collect();
        return taskbook.delete_items(&ids);
    }

    if check {
        let ids: Vec<u64> = input.iter().filter_map(|s| s.parse().ok()).collect();
        return taskbook.check_tasks(&ids);
    }

    if begin {
        let ids: Vec<u64> = input.iter().filter_map(|s| s.parse().ok()).collect();
        return taskbook.begin_tasks(&ids);
    }

    if star {
        let ids: Vec<u64> = input.iter().filter_map(|s| s.parse().ok()).collect();
        return taskbook.star_items(&ids);
    }

    if priority {
        return taskbook.update_priority(&input);
    }

    if copy {
        let ids: Vec<u64> = input.iter().filter_map(|s| s.parse().ok()).collect();
        return taskbook.copy_to_clipboard(&ids);
    }

    if timeline {
        taskbook.display_by_date()?;
        return taskbook.display_stats();
    }

    if find {
        return taskbook.find_items(&input);
    }

    if list {
        taskbook.list_by_attributes(&input)?;
        return taskbook.display_stats();
    }

    if edit {
        return taskbook.edit_description(&input);
    }

    if r#move {
        return taskbook.move_boards(&input);
    }

    if clear {
        return taskbook.clear();
    }

    // Default: display board view and stats
    taskbook.display_by_board()?;
    taskbook.display_stats()
}

/// Migrate local data to the remote server.
pub fn migrate(taskbook_dir: Option<PathBuf>) -> Result<()> {
    let creds = Credentials::load()?.ok_or_else(|| {
        TaskbookError::Auth("not logged in — run `tb register` or `tb login` first".to_string())
    })?;

    let config = Config::load().unwrap_or_default();
    let encryption_key = creds.encryption_key_bytes()?;
    let engine = base64::engine::general_purpose::STANDARD;

    // Load local data
    let resolved_dir = resolve_taskbook_directory(taskbook_dir.as_deref())?;
    let local = LocalStorage::new(&resolved_dir)?;

    let items = local.get()?;
    let archive = local.get_archive()?;

    // Encrypt and upload items
    let client = ApiClient::new(&config.sync.server_url, Some(&creds.token));

    let mut encrypted_items = std::collections::HashMap::new();
    for (key, item) in &items {
        let encrypted = encrypt_item(&encryption_key, item)
            .map_err(|e| TaskbookError::General(format!("encryption failed: {e}")))?;
        encrypted_items.insert(
            key.clone(),
            EncryptedItemData {
                data: engine.encode(&encrypted.data),
                nonce: engine.encode(&encrypted.nonce),
            },
        );
    }
    client.put_items(&encrypted_items)?;

    let mut encrypted_archive = std::collections::HashMap::new();
    for (key, item) in &archive {
        let encrypted = encrypt_item(&encryption_key, item)
            .map_err(|e| TaskbookError::General(format!("encryption failed: {e}")))?;
        encrypted_archive.insert(
            key.clone(),
            EncryptedItemData {
                data: engine.encode(&encrypted.data),
                nonce: engine.encode(&encrypted.nonce),
            },
        );
    }
    client.put_archive(&encrypted_archive)?;

    println!(
        "{}",
        format!(
            "Migrated {} items and {} archived items to server.",
            items.len(),
            archive.len()
        )
        .green()
        .bold()
    );
    println!(
        "{}",
        "To enable sync, set sync.enabled = true in ~/.taskbook.json".dimmed()
    );

    Ok(())
}
