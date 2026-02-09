use std::env;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::error::{Result, TaskbookError};

const TASKBOOK_DIR_NAME: &str = ".taskbook";
const TASKBOOK_DIR_ENV: &str = "TASKBOOK_DIR";

fn home_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .ok_or_else(|| TaskbookError::General("could not find home directory".to_string()))
}

/// Resolve the taskbook directory with priority:
/// 1. --taskbook-dir CLI flag (highest)
/// 2. TASKBOOK_DIR environment variable
/// 3. Config file taskbookDirectory
/// 4. Default ~/.taskbook/ (lowest)
pub fn resolve_taskbook_directory(cli_taskbook_dir: Option<&Path>) -> Result<PathBuf> {
    // Try to resolve a custom directory
    if let Some(custom_dir) = resolve_custom_directory(cli_taskbook_dir)? {
        return Ok(custom_dir);
    }

    // Default to ~/.taskbook/
    let home = home_dir()?;
    Ok(home.join(TASKBOOK_DIR_NAME))
}

fn resolve_custom_directory(cli_taskbook_dir: Option<&Path>) -> Result<Option<PathBuf>> {
    let candidate = select_custom_directory_candidate(cli_taskbook_dir)?;

    let candidate = match candidate {
        Some(c) => c,
        None => return Ok(None),
    };

    let resolved = parse_directory(&candidate);

    // Check if the candidate path ends with .taskbook
    if is_taskbook_directory_path(&resolved) {
        let parent = resolved.parent().ok_or_else(|| {
            TaskbookError::InvalidDirectory(format!("{candidate}: path has no parent"))
        })?;
        assert_directory_exists(parent, &candidate)?;
        return Ok(Some(resolved));
    }

    assert_directory_exists(&resolved, &candidate)?;
    Ok(Some(resolved.join(TASKBOOK_DIR_NAME)))
}

fn select_custom_directory_candidate(cli_taskbook_dir: Option<&Path>) -> Result<Option<String>> {
    // Priority 1: CLI flag
    if let Some(dir) = cli_taskbook_dir {
        let dir_str = dir.to_string_lossy().to_string();
        if dir_str.trim().is_empty() {
            return Err(TaskbookError::MissingTaskbookDirValue);
        }
        return Ok(Some(dir_str));
    }

    // Priority 2: Environment variable
    if let Ok(env_dir) = env::var(TASKBOOK_DIR_ENV) {
        if !env_dir.trim().is_empty() {
            return Ok(Some(env_dir));
        }
    }

    // Priority 3: Config file
    if let Ok(config) = Config::load() {
        let config_dir = &config.taskbook_directory;
        // Only use config dir if it's not the default home directory
        let home = home_dir()?.to_string_lossy().to_string();
        if config_dir != &home && config_dir != "~" {
            return Ok(Some(config_dir.clone()));
        }
    }

    Ok(None)
}

fn parse_directory(directory: &str) -> PathBuf {
    let expanded = expand_directory(directory);
    PathBuf::from(&expanded)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&expanded))
}

fn expand_directory(directory: &str) -> String {
    if directory.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            let rest = directory.trim_start_matches('~');
            return format!("{}{}", home.to_string_lossy(), rest);
        }
    }
    directory.to_string()
}

fn is_taskbook_directory_path(path: &Path) -> bool {
    path.file_name()
        .map(|name| name == TASKBOOK_DIR_NAME)
        .unwrap_or(false)
}

fn assert_directory_exists(directory: &Path, display_path: &str) -> Result<()> {
    let expanded = if display_path.starts_with('~') {
        PathBuf::from(expand_directory(display_path))
    } else {
        PathBuf::from(display_path)
    };

    // Check if directory exists - if expanded path exists or the resolved path exists
    if expanded.exists() || directory.exists() {
        return Ok(());
    }

    Err(TaskbookError::InvalidDirectory(display_path.to_string()))
}
