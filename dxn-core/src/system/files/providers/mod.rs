//! File Storage Providers
//! 
//! This module provides an abstraction layer for file storage operations.
//! Different providers (local, SFTP, S3, etc.) implement the FileProvider trait
//! to enable flexible file storage across different backends.

pub mod local;

use std::collections::HashMap;
use std::sync::RwLock;
use crate::data::models::FileEntry;

// ============================================================================
// PROVIDER ERROR
// ============================================================================

/// Errors that can occur during file provider operations
#[derive(Debug)]
pub enum ProviderError {
    /// File or directory not found
    NotFound(String),
    /// Permission denied
    PermissionDenied(String),
    /// Path already exists
    AlreadyExists(String),
    /// IO error occurred
    IoError(std::io::Error),
    /// Provider not configured or unavailable
    ProviderUnavailable(String),
    /// Invalid path or path traversal attempt
    InvalidPath(String),
    /// Other errors
    Other(String),
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderError::NotFound(p) => write!(f, "Not found: {}", p),
            ProviderError::PermissionDenied(p) => write!(f, "Permission denied: {}", p),
            ProviderError::AlreadyExists(p) => write!(f, "Already exists: {}", p),
            ProviderError::IoError(e) => write!(f, "IO error: {}", e),
            ProviderError::ProviderUnavailable(p) => write!(f, "Provider unavailable: {}", p),
            ProviderError::InvalidPath(p) => write!(f, "Invalid path: {}", p),
            ProviderError::Other(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for ProviderError {}

impl From<std::io::Error> for ProviderError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => ProviderError::NotFound(err.to_string()),
            std::io::ErrorKind::PermissionDenied => ProviderError::PermissionDenied(err.to_string()),
            std::io::ErrorKind::AlreadyExists => ProviderError::AlreadyExists(err.to_string()),
            _ => ProviderError::IoError(err),
        }
    }
}

// ============================================================================
// FILE PROVIDER TRAIT
// ============================================================================

/// Trait for file storage providers
/// 
/// Providers implement this trait to enable file operations across different
/// storage backends (local filesystem, SFTP, S3, etc.)
pub trait FileProvider: Send + Sync {
    /// List files and directories at the given path
    /// 
    /// # Arguments
    /// * `path` - Path relative to the provider's base path
    /// 
    /// # Returns
    /// Vector of file entries, or error
    fn list(&self, path: &str) -> Result<Vec<FileEntry>, ProviderError>;
    
    /// Read file contents as bytes
    /// 
    /// # Arguments
    /// * `path` - Path to the file relative to the provider's base path
    /// 
    /// # Returns
    /// File contents as bytes, or error
    fn read(&self, path: &str) -> Result<Vec<u8>, ProviderError>;
    
    /// Read file contents as string (UTF-8)
    /// 
    /// # Arguments
    /// * `path` - Path to the file relative to the provider's base path
    /// 
    /// # Returns
    /// File contents as string, or error
    fn read_string(&self, path: &str) -> Result<String, ProviderError> {
        let bytes = self.read(path)?;
        String::from_utf8(bytes)
            .map_err(|e| ProviderError::Other(format!("Invalid UTF-8: {}", e)))
    }
    
    /// Write bytes to a file (creates or overwrites)
    /// 
    /// # Arguments
    /// * `path` - Path to the file relative to the provider's base path
    /// * `contents` - Bytes to write
    /// 
    /// # Returns
    /// Ok(()) on success, or error
    fn write(&self, path: &str, contents: &[u8]) -> Result<(), ProviderError>;
    
    /// Write string to a file (creates or overwrites)
    /// 
    /// # Arguments
    /// * `path` - Path to the file relative to the provider's base path
    /// * `contents` - String to write
    /// 
    /// # Returns
    /// Ok(()) on success, or error
    fn write_string(&self, path: &str, contents: &str) -> Result<(), ProviderError> {
        self.write(path, contents.as_bytes())
    }
    
    /// Append bytes to a file
    /// 
    /// # Arguments
    /// * `path` - Path to the file relative to the provider's base path
    /// * `contents` - Bytes to append
    /// 
    /// # Returns
    /// Ok(()) on success, or error
    fn append(&self, path: &str, contents: &[u8]) -> Result<(), ProviderError>;
    
    /// Delete a file or empty directory
    /// 
    /// # Arguments
    /// * `path` - Path to the file/directory relative to the provider's base path
    /// 
    /// # Returns
    /// Ok(()) on success, or error
    fn delete(&self, path: &str) -> Result<(), ProviderError>;
    
    /// Check if a path exists
    /// 
    /// # Arguments
    /// * `path` - Path to check relative to the provider's base path
    /// 
    /// # Returns
    /// true if exists, false otherwise
    fn exists(&self, path: &str) -> Result<bool, ProviderError>;
    
    /// Create a directory (and parent directories if needed)
    /// 
    /// # Arguments
    /// * `path` - Path to the directory relative to the provider's base path
    /// 
    /// # Returns
    /// Ok(()) on success, or error
    fn mkdir(&self, path: &str) -> Result<(), ProviderError>;
    
    /// Get file/directory metadata
    /// 
    /// # Arguments
    /// * `path` - Path to the file/directory relative to the provider's base path
    /// 
    /// # Returns
    /// FileEntry with metadata, or error
    fn metadata(&self, path: &str) -> Result<FileEntry, ProviderError>;
}

// ============================================================================
// PROVIDER REGISTRY
// ============================================================================

/// Registry for file providers
/// 
/// Manages registered providers and provides access to them by name
pub struct ProviderRegistry {
    providers: RwLock<HashMap<String, Box<dyn FileProvider>>>,
}

impl ProviderRegistry {
    /// Create a new empty provider registry
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a provider with the given name
    /// 
    /// # Arguments
    /// * `name` - Unique name for this provider instance
    /// * `provider` - The provider implementation
    pub fn register(&self, name: &str, provider: Box<dyn FileProvider>) {
        let mut providers = self.providers.write().unwrap();
        providers.insert(name.to_string(), provider);
    }
    
    /// Unregister a provider by name
    /// 
    /// # Arguments
    /// * `name` - Name of the provider to remove
    pub fn unregister(&self, name: &str) {
        let mut providers = self.providers.write().unwrap();
        providers.remove(name);
    }
    
    /// Check if a provider is registered
    /// 
    /// # Arguments
    /// * `name` - Name of the provider to check
    /// 
    /// # Returns
    /// true if registered, false otherwise
    pub fn has(&self, name: &str) -> bool {
        let providers = self.providers.read().unwrap();
        providers.contains_key(name)
    }
    
    /// Get a reference to a provider by name and execute a closure with it
    /// 
    /// # Arguments
    /// * `name` - Name of the provider
    /// * `f` - Closure to execute with the provider
    /// 
    /// # Returns
    /// Result of the closure, or ProviderUnavailable error
    pub fn with_provider<F, R>(&self, name: &str, f: F) -> Result<R, ProviderError>
    where
        F: FnOnce(&dyn FileProvider) -> Result<R, ProviderError>,
    {
        let providers = self.providers.read().unwrap();
        match providers.get(name) {
            Some(provider) => f(provider.as_ref()),
            None => Err(ProviderError::ProviderUnavailable(name.to_string())),
        }
    }
    
    /// List all registered provider names
    /// 
    /// # Returns
    /// Vector of provider names
    pub fn list_providers(&self) -> Vec<String> {
        let providers = self.providers.read().unwrap();
        providers.keys().cloned().collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
