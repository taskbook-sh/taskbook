use std::path::PathBuf;

use crate::error::Result;
use crate::taskbook::Taskbook;

/// Execute CLI commands
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
        return taskbook.create_note(&input);
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
