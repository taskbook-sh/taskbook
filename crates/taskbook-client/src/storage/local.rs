use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::error::Result;
use taskbook_common::StorageItem;

use super::StorageBackend;

/// Local file-based storage with atomic writes
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
            .unwrap()
            .to_string_lossy()
            .to_string();
        let temp_filename = filename.replace(".json", &format!(".TEMP-{}.json", random_string));
        self.temp_dir.join(temp_filename)
    }
}

impl StorageBackend for LocalStorage {
    fn get(&self) -> Result<HashMap<String, StorageItem>> {
        if !self.storage_file.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&self.storage_file)?;
        let data: HashMap<String, StorageItem> = serde_json::from_str(&content)?;
        Ok(data)
    }

    fn get_archive(&self) -> Result<HashMap<String, StorageItem>> {
        if !self.archive_file.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&self.archive_file)?;
        let data: HashMap<String, StorageItem> = serde_json::from_str(&content)?;
        Ok(data)
    }

    fn set(&self, data: &HashMap<String, StorageItem>) -> Result<()> {
        let json = serde_json::to_string_pretty(data)?;
        let temp_file = self.get_temp_file(&self.storage_file);

        fs::write(&temp_file, json)?;
        fs::rename(&temp_file, &self.storage_file)?;

        Ok(())
    }

    fn set_archive(&self, data: &HashMap<String, StorageItem>) -> Result<()> {
        let json = serde_json::to_string_pretty(data)?;
        let temp_file = self.get_temp_file(&self.archive_file);

        fs::write(&temp_file, json)?;
        fs::rename(&temp_file, &self.archive_file)?;

        Ok(())
    }
}
