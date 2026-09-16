use std::fs;
use std::path::{Path, PathBuf};

use crate::core::SavedRequest;

use super::{RequestRepository, StorageError};

pub struct JsonFileRequestRepository {
    path: PathBuf,
}

impl JsonFileRequestRepository {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into()
        }
    }

    pub fn with_default_path() -> Self {
        let dir: PathBuf = dirs::data_local_dir().unwrap_or_else (|| PathBuf::from("."));
        Self::new(dir.join("songbird").join("requests.json"))
    }

    fn read_entries(path: &Path) -> Result<Vec<SavedRequest>, StorageError> {
        if !path.exists() {
            return Ok(Vec::new());
        }

        let raw: String = fs::read_to_string(path)?;
        if raw.trim().is_empty() {
            return Ok(Vec::new());
        }

        Ok(serde_json::from_str(&raw)?)
    }

    fn write_entries(path: &Path, entries: &[SavedRequest]) -> Result<(), StorageError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let raw = serde_json::to_string_pretty(entries)?;
        fs::write(path, format!("{raw}\n"))?;
        Ok(())
    }
}

impl RequestRepository for JsonFileRequestRepository {
    fn load_all(&self) -> Result<Vec<SavedRequest>, StorageError> {
        Self::read_entries(&self.path)
    }

    fn save(&self, entry: &SavedRequest) -> Result<(), StorageError> {
        let mut entries = Self::read_entries(&self.path)?;
        entries.retain(|existing| existing.id != entry.id);
        entries.push(entry.clone());
        Self::write_entries(&self.path, &entries)
    }

    fn delete(&self, id: &str) -> Result<(), StorageError> {
        let mut entries = Self::read_entries(&self.path)?;
        entries.retain(|existing| existing.id != id);
        Self::write_entries(&self.path, &entries)
    }
}
