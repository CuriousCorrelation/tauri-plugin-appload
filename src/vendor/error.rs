use thiserror::Error;

#[derive(Debug, Error)]
pub enum VendorError {
    #[error("Missing vendor configuration: {0}")]
    MissingConfig(String),

    #[error("Invalid vendor data: {0}")]
    InvalidData(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("Cache error: {0}")]
    Cache(#[from] crate::cache::CacheError),

    #[error("Bundle error: {0}")]
    Bundle(#[from] crate::bundle::BundleError),

    #[error("Storage error: {0}")]
    Storage(#[from] crate::storage::StorageError),
}

pub type Result<T> = std::result::Result<T, VendorError>;
