mod json_file;

use thiserror::Error;

use crate::core::SavedRequest;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("failed to read request store: {0}")]
    Read(#[from] std::io::Error),
    #[error("request store is corrupted: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("request store lock is poisoned")]
    Poisoned,
}

pub trait RequestRepository: Send + Sync {
    fn load_all(&self) -> Result<Vec<SavedRequest>, StorageError>;
    fn save(&self, entry: &SavedRequest) -> Result<(), StorageError>;
    fn delete(&self, id: &str) -> Result<(), StorageError>;
}

pub use json_file::JsonFileRequestRepository;
