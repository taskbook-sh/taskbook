use std::collections::HashMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use fs2::FileExt;
use uuid::Uuid;

use crate::error::Result;
use taskbook_common::StorageItem;

use super::StorageBackend;

/// Local file-based storage with atomic writes and file locking
pub struct LocalStorage {
    main_app_dir: PathBuf,
    storage_dir: PathBuf,
    archive_dir: PathBuf,
    temp_dir: PathBuf,
    storage_file: PathBuf,
    archive_file: PathBuf,
}

impl LocalStorage {
    pub fn new(taskbook_dir: &Path) -> Result<Self> {
        let main_app_dir = taskbook_dir.to_path_buf();
        let storage_dir = main_app_dir.join("storage");
        let archive_dir = main_app_dir.join("archive");
        let temp_dir = main_app_dir.join(".temp");
        let storage_file = storage_dir.join("storage.json");
        let archive_file = archive_dir.join("archive.json");

        let storage = Self {
            main_app_dir,
            storage_dir,
            archive_dir,
            temp_dir,
            storage_file,
            archive_file,
        };

        storage.ensure_directories()?;

        Ok(storage)
    }

    fn ensure_directories(&self) -> Result<()> {
        if !self.main_app_dir.exists() {
            fs::create_dir_all(&self.main_app_dir)?;
        }
        if !self.storage_dir.exists() {
            fs::create_dir(&self.storage_dir)?;
        }
        if !self.archive_dir.exists() {
            fs::create_dir(&self.archive_dir)?;
        }
        if !self.temp_dir.exists() {
            fs::create_dir(&self.temp_dir)?;
        }

        self.clean_temp_dir()?;

        Ok(())
    }

    fn clean_temp_dir(&self) -> Result<()> {
        if self.temp_dir.exists() {
            for entry in fs::read_dir(&self.temp_dir)? {
                let entry = entry?;
                fs::remove_file(entry.path())?;
            }
        }
        Ok(())
    }

    fn get_temp_file(&self, target_file: &Path) -> PathBuf {
        let random_string = Uuid::new_v4().to_string()[..8].to_string();
        let filename = target_file
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let temp_filename = filename.replace(".json", &format!(".TEMP-{}.json", random_string));
        self.temp_dir.join(temp_filename)
    }

    /// Acquire an exclusive lock on the given file path.
    /// Creates the lock file if it doesn't exist.
    fn lock_file(&self, path: &Path) -> Result<File> {
        let lock_path = path.with_extension("lock");
        let lock_file = File::create(&lock_path)?;
        lock_file.lock_exclusive()?;
        Ok(lock_file)
    }

    fn read_json_file(&self, path: &Path) -> Result<HashMap<String, StorageItem>> {
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let content = fs::read_to_string(path)?;
        let data: HashMap<String, StorageItem> = serde_json::from_str(&content)?;
        Ok(data)
    }

    fn write_json_file(
        &self,
        path: &Path,
        data: &HashMap<String, StorageItem>,
    ) -> Result<()> {
        let json = serde_json::to_string_pretty(data)?;
        let temp_file = self.get_temp_file(path);
        fs::write(&temp_file, json)?;
        fs::rename(&temp_file, path)?;
        Ok(())
    }
}

impl StorageBackend for LocalStorage {
    fn get(&self) -> Result<HashMap<String, StorageItem>> {
        let _lock = self.lock_file(&self.storage_file)?;
        self.read_json_file(&self.storage_file)
    }

    fn get_archive(&self) -> Result<HashMap<String, StorageItem>> {
        let _lock = self.lock_file(&self.archive_file)?;
        self.read_json_file(&self.archive_file)
    }

    fn set(&self, data: &HashMap<String, StorageItem>) -> Result<()> {
        let _lock = self.lock_file(&self.storage_file)?;
        self.write_json_file(&self.storage_file, data)
    }

    fn set_archive(&self, data: &HashMap<String, StorageItem>) -> Result<()> {
        let _lock = self.lock_file(&self.archive_file)?;
        self.write_json_file(&self.archive_file, data)
    }
}
