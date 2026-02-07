//! Local File System Provider
//! 
//! Provides file operations on the local filesystem.
//! Files are stored relative to a configurable base path.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};

use crate::data::models::FileEntry;
use super::{FileProvider, ProviderError};

/// Default base path for file storage
const DEFAULT_BASE_PATH: &str = "../dxn-files";

// ============================================================================
// LOCAL FILE PROVIDER
// ============================================================================

/// Local filesystem provider
/// 
/// Stores files on the local filesystem relative to a base path.
/// Includes path traversal protection to prevent escaping the base directory.
pub struct LocalFileProvider {
    /// Base path for all file operations
    base_path: PathBuf,
}

impl LocalFileProvider {
    /// Create a new local file provider with the specified base path
    /// 
    /// # Arguments
    /// * `base_path` - Base directory for file storage
    /// 
    /// # Returns
    /// New LocalFileProvider instance
    pub fn new(base_path: &str) -> Self {
        let path = if base_path.is_empty() {
            PathBuf::from(DEFAULT_BASE_PATH)
        } else {
            PathBuf::from(base_path)
        };
        
        Self { base_path: path }
    }

    /// Create a local file provider whose base path must lie inside `project_root`.
    /// Rejects base paths that would allow operations outside the project (e.g. `../` escaping).
    ///
    /// # Arguments
    /// * `base_path` - Base directory for file storage (relative or absolute)
    /// * `project_root` - Absolute path of the project root; resolved base must be under this
    ///
    /// # Returns
    /// `Ok(Self)` if base path is inside project root, `Err(ProviderError::InvalidPath)` otherwise.
    pub fn new_with_project_root(
        base_path: &str,
        project_root: &str,
    ) -> Result<Self, ProviderError> {
        let root_buf = PathBuf::from(project_root);
        let project_root_canonical = root_buf
            .canonicalize()
            .map_err(ProviderError::IoError)?;

        let base_path_buf = PathBuf::from(if base_path.is_empty() {
            DEFAULT_BASE_PATH
        } else {
            base_path
        });
        let base_absolute = if base_path_buf.is_relative() {
            std::env::current_dir()
                .map_err(ProviderError::IoError)?
                .join(base_path_buf)
        } else {
            base_path_buf
        };

        // Normalize: resolve . and .. so we can check it stays under project root
        let mut resolved = PathBuf::new();
        for comp in base_absolute.components() {
            match comp {
                std::path::Component::Prefix(p) => resolved.push(p.as_os_str()),
                std::path::Component::RootDir => resolved.push(std::path::MAIN_SEPARATOR_STR),
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    if !resolved.pop() {
                        return Err(ProviderError::InvalidPath(
                            "Path escapes project root".to_string(),
                        ));
                    }
                }
                std::path::Component::Normal(n) => resolved.push(n),
            }
        }

        if !resolved.starts_with(&project_root_canonical) {
            return Err(ProviderError::InvalidPath(format!(
                "File directory path is outside project root: {}",
                resolved.display()
            )));
        }

        Ok(Self {
            base_path: resolved,
        })
    }
    
    /// Create a provider with the default base path
    pub fn default() -> Self {
        Self::new(DEFAULT_BASE_PATH)
    }
    
    /// Get the full path for a relative path, with path traversal protection
    /// 
    /// # Arguments
    /// * `relative_path` - Path relative to the base
    /// 
    /// # Returns
    /// Full path, or error if path traversal is detected
    fn get_full_path(&self, relative_path: &str) -> Result<PathBuf, ProviderError> {
        // Normalize the path by removing leading slashes
        let normalized = relative_path.trim_start_matches('/');
        
        // Build the full path
        let full_path = self.base_path.join(normalized);
        
        // Canonicalize both paths for comparison (if they exist)
        // For non-existent paths, we check component by component
        let canonical_base = self.base_path.canonicalize()
            .unwrap_or_else(|_| self.base_path.clone());
        
        // Check that the path doesn't escape the base directory
        // We need to check even for non-existent paths
        let mut check_path = self.base_path.clone();
        for component in Path::new(normalized).components() {
            match component {
                std::path::Component::ParentDir => {
                    // Going up - check if we're still within base
                    if !check_path.pop() || !check_path.starts_with(&canonical_base) {
                        return Err(ProviderError::InvalidPath(
                            format!("Path traversal detected: {}", relative_path)
                        ));
                    }
                }
                std::path::Component::Normal(name) => {
                    check_path.push(name);
                }
                std::path::Component::CurDir => {
                    // Current dir (.) - ignore
                }
                _ => {
                    // Root or prefix - not allowed in relative paths
                    return Err(ProviderError::InvalidPath(
                        format!("Invalid path component: {}", relative_path)
                    ));
                }
            }
        }
        
        Ok(full_path)
    }
    
    /// Get MIME type based on file extension
    fn get_mime_type(path: &Path) -> Option<String> {
        let extension = path.extension()?.to_str()?.to_lowercase();
        
        let mime = match extension.as_str() {
            // Text
            "txt" => "text/plain",
            "html" | "htm" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "json" => "application/json",
            "xml" => "application/xml",
            "csv" => "text/csv",
            "md" => "text/markdown",
            
            // Images
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "webp" => "image/webp",
            "ico" => "image/x-icon",
            
            // Documents
            "pdf" => "application/pdf",
            "doc" => "application/msword",
            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "xls" => "application/vnd.ms-excel",
            "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            
            // Archives
            "zip" => "application/zip",
            "tar" => "application/x-tar",
            "gz" => "application/gzip",
            
            // Audio/Video
            "mp3" => "audio/mpeg",
            "mp4" => "video/mp4",
            "webm" => "video/webm",
            
            // Other
            "wasm" => "application/wasm",
            
            _ => "application/octet-stream",
        };
        
        Some(mime.to_string())
    }
}

impl FileProvider for LocalFileProvider {
    fn list(&self, path: &str) -> Result<Vec<FileEntry>, ProviderError> {
        let full_path = self.get_full_path(path)?;
        
        if !full_path.exists() {
            return Err(ProviderError::NotFound(path.to_string()));
        }
        
        if !full_path.is_dir() {
            return Err(ProviderError::InvalidPath(
                format!("Not a directory: {}", path)
            ));
        }
        
        let mut entries = Vec::new();
        
        for entry in fs::read_dir(&full_path)? {
            let entry = entry?;
            let entry_path = entry.path();
            let metadata = entry.metadata()?;
            
            let name = entry.file_name().to_string_lossy().to_string();
            let is_directory = metadata.is_dir();
            
            // Build relative path from the requested directory
            let relative_path = if path.is_empty() || path == "/" {
                name.clone()
            } else {
                format!("{}/{}", path.trim_end_matches('/'), name)
            };
            
            let modified = metadata.modified().ok().map(|t| {
                let datetime: DateTime<Utc> = t.into();
                datetime.to_rfc3339()
            });
            
            let size = if is_directory { None } else { Some(metadata.len()) };
            let mime_type = if is_directory { None } else { Self::get_mime_type(&entry_path) };
            
            entries.push(FileEntry {
                name,
                path: relative_path,
                is_directory,
                size,
                modified,
                mime_type,
            });
        }
        
        // Sort: directories first, then by name
        entries.sort_by(|a, b| {
            match (a.is_directory, b.is_directory) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });
        
        Ok(entries)
    }
    
    fn read(&self, path: &str) -> Result<Vec<u8>, ProviderError> {
        let full_path = self.get_full_path(path)?;
        
        if !full_path.exists() {
            return Err(ProviderError::NotFound(path.to_string()));
        }
        
        if full_path.is_dir() {
            return Err(ProviderError::InvalidPath(
                format!("Cannot read directory as file: {}", path)
            ));
        }
        
        let mut file = File::open(&full_path)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        
        Ok(contents)
    }
    
    fn write(&self, path: &str, contents: &[u8]) -> Result<(), ProviderError> {
        let full_path = self.get_full_path(path)?;
        
        // Create parent directories if they don't exist
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let mut file = File::create(&full_path)?;
        file.write_all(contents)?;
        file.sync_all()?;
        
        Ok(())
    }
    
    fn append(&self, path: &str, contents: &[u8]) -> Result<(), ProviderError> {
        let full_path = self.get_full_path(path)?;
        
        // Create parent directories if they don't exist
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&full_path)?;
        
        file.write_all(contents)?;
        file.sync_all()?;
        
        Ok(())
    }
    
    fn delete(&self, path: &str) -> Result<(), ProviderError> {
        let full_path = self.get_full_path(path)?;
        
        if !full_path.exists() {
            return Err(ProviderError::NotFound(path.to_string()));
        }
        
        if full_path.is_dir() {
            fs::remove_dir(&full_path)?;
        } else {
            fs::remove_file(&full_path)?;
        }
        
        Ok(())
    }
    
    fn exists(&self, path: &str) -> Result<bool, ProviderError> {
        let full_path = self.get_full_path(path)?;
        Ok(full_path.exists())
    }
    
    fn mkdir(&self, path: &str) -> Result<(), ProviderError> {
        let full_path = self.get_full_path(path)?;
        fs::create_dir_all(&full_path)?;
        Ok(())
    }
    
    fn metadata(&self, path: &str) -> Result<FileEntry, ProviderError> {
        let full_path = self.get_full_path(path)?;
        
        if !full_path.exists() {
            return Err(ProviderError::NotFound(path.to_string()));
        }
        
        let metadata = fs::metadata(&full_path)?;
        let is_directory = metadata.is_dir();
        
        let name = full_path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());
        
        let modified = metadata.modified().ok().map(|t| {
            let datetime: DateTime<Utc> = t.into();
            datetime.to_rfc3339()
        });
        
        let size = if is_directory { None } else { Some(metadata.len()) };
        let mime_type = if is_directory { None } else { Self::get_mime_type(&full_path) };
        
        Ok(FileEntry {
            name,
            path: path.to_string(),
            is_directory,
            size,
            modified,
            mime_type,
        })
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    fn create_test_provider() -> (LocalFileProvider, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let provider = LocalFileProvider::new(temp_dir.path().to_str().unwrap());
        (provider, temp_dir)
    }
    
    #[test]
    fn test_write_and_read() {
        let (provider, _temp) = create_test_provider();
        
        let content = b"Hello, World!";
        provider.write("test.txt", content).unwrap();
        
        let read_content = provider.read("test.txt").unwrap();
        assert_eq!(read_content, content);
    }
    
    #[test]
    fn test_list_directory() {
        let (provider, _temp) = create_test_provider();
        
        // Create some files and directories
        provider.mkdir("subdir").unwrap();
        provider.write("file1.txt", b"content1").unwrap();
        provider.write("file2.txt", b"content2").unwrap();
        provider.write("subdir/nested.txt", b"nested").unwrap();
        
        let entries = provider.list("").unwrap();
        
        assert_eq!(entries.len(), 3);
        
        // Directories should come first
        assert!(entries[0].is_directory);
        assert_eq!(entries[0].name, "subdir");
        
        // Then files alphabetically
        assert!(!entries[1].is_directory);
        assert!(!entries[2].is_directory);
    }
    
    #[test]
    fn test_path_traversal_protection() {
        let (provider, _temp) = create_test_provider();
        
        // Attempting to escape should fail
        let result = provider.read("../../../etc/passwd");
        assert!(result.is_err());
        
        match result {
            Err(ProviderError::InvalidPath(_)) => {}
            _ => panic!("Expected InvalidPath error"),
        }
    }
    
    #[test]
    fn test_delete_file() {
        let (provider, _temp) = create_test_provider();
        
        provider.write("to_delete.txt", b"delete me").unwrap();
        assert!(provider.exists("to_delete.txt").unwrap());
        
        provider.delete("to_delete.txt").unwrap();
        assert!(!provider.exists("to_delete.txt").unwrap());
    }
    
    #[test]
    fn test_append() {
        let (provider, _temp) = create_test_provider();
        
        provider.write("append.txt", b"Hello").unwrap();
        provider.append("append.txt", b", World!").unwrap();
        
        let content = provider.read_string("append.txt").unwrap();
        assert_eq!(content, "Hello, World!");
    }
}
