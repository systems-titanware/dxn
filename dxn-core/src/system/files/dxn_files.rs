//! Single entry point for resolving and reading files under `project_root/dxn-files/`.
//! All path resolution uses the project root; no cwd-relative paths.

use std::path::PathBuf;

use super::providers::local::LocalFileProvider;
use super::providers::{FileProvider, ProviderError};

/// Directory name under project root for dxn file storage (routes, _files, etc.).
pub const DXN_FILES_DIR: &str = "dxn-files";

/// Resolves a path under `project_root/dxn-files/` with traversal protection.
/// Use for getting an absolute path to pass to other APIs (e.g. migrations, executors).
pub fn resolve_under_dxn_files(
    project_root: &str,
    relative_path: &str,
) -> Result<PathBuf, ProviderError> {
    let base = format!("{}/{}", project_root.trim_end_matches('/'), DXN_FILES_DIR);
    let provider = LocalFileProvider::new(&base);
    provider.resolve_path(relative_path)
}

/// Reads a file under `project_root/dxn-files/` as a string.
/// Uses the same LocalFileProvider for consistency and path safety.
pub fn read_under_dxn_files(
    project_root: &str,
    relative_path: &str,
) -> Result<String, ProviderError> {
    let base = format!("{}/{}", project_root.trim_end_matches('/'), DXN_FILES_DIR);
    let provider = LocalFileProvider::new(&base);
    provider.read_string(relative_path)
}

/// Resolves a path under an already-resolved absolute `dxn-files` root.
/// This avoids rebuilding base paths repeatedly in request handlers.
pub fn resolve_under_dxn_files_root(
    dxn_files_root: &str,
    relative_path: &str,
) -> Result<PathBuf, ProviderError> {
    let provider = LocalFileProvider::new(dxn_files_root);
    provider.resolve_path(relative_path)
}

/// Reads a file under an already-resolved absolute `dxn-files` root.
/// Uses startup-cached root so no project-root discovery is performed per read.
pub fn read_under_dxn_files_root(
    dxn_files_root: &str,
    relative_path: &str,
) -> Result<String, ProviderError> {
    let provider = LocalFileProvider::new(dxn_files_root);
    provider.read_string(relative_path)
}
