mod local;
mod remote;

pub use local::LocalStorage;
pub use remote::RemoteStorage;

use std::collections::HashMap;

use crate::error::Result;
use taskbook_common::StorageItem;

/// Trait abstracting storage backends (local file, remote server, etc.)
pub trait StorageBackend {
    fn get(&self) -> Result<HashMap<String, StorageItem>>;
    fn get_archive(&self) -> Result<HashMap<String, StorageItem>>;
    fn set(&self, data: &HashMap<String, StorageItem>) -> Result<()>;
    fn set_archive(&self, data: &HashMap<String, StorageItem>) -> Result<()>;
}
