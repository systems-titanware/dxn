#![feature(concat_bytes)] // Required for std::concat_bytes!

use std::fs;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::prelude::*;
#[cfg(target_family = "unix")]
use std::os::unix;
#[cfg(target_family = "windows")]
use std::os::windows;
#[cfg(target_family = "darwin")]
use std::os::darwin;
use std::path::Path;
use std::path::PathBuf;
use std::io::Write;
use std::sync::OnceLock;

/// Subdir under dxn-files used by the file manager (e.g. logs).
const FILES_SUBDIR: &str = "_files";
/// Fallback when project root not set (relative to cwd).
const ROOT_FILE_PATH_FALLBACK: &str = "../dxn-files/_files";

static PROJECT_ROOT: OnceLock<String> = OnceLock::new();

/// Set the project root at startup so get_full_path uses project_root/dxn-files/_files.
pub fn set_project_root(root: &str) {
    let _ = PROJECT_ROOT.set(root.trim_end_matches('/').to_string());
}

// A simple implementation of `% cat path`
pub fn read_file(path: &str) -> io::Result<String> {
    let full_path = get_full_path(path);
    fs::read_to_string(&full_path)
}

// A simple implementation of `% echo s >> path` (append)
// Sync guaranteed: Uses sync_all() to ensure data is flushed to disk before returning
pub fn add_content(str: &str, path: &str) -> io::Result<()> {
    let full_path = get_full_path(path);
    // Open a file with append option.
    // .append(true) ensures new data is added to the end of the file.
    // .create(true) will create the file if it doesn't exist.
    let mut file = OpenOptions::new()
        .append(true)
        .create(true) // Create the file if it doesn't exist
        .open(full_path)?; // Use `?` for concise error propagation

    // Write content to the file.
    // .write_all() takes a byte slice.
    file.write_all(str.as_bytes())?;
    file.sync_all()?;  // Forces flush to disk (not just OS buffers)
    
    Ok(())
}

// A simple implementation of `% echo s > path`
// Sync guaranteed: Uses sync_all() to ensure data is flushed to disk before returning
pub fn add_file_content(s: &str, path: &str) -> io::Result<()> {
    let full_path = get_full_path(path);
    // Ensure parent directory exists
    if let Some(parent) = full_path.parent() {
        // If parent exists as a file, remove it first (shouldn't happen, but handle it)
        if parent.exists() && parent.is_file() {
            fs::remove_file(parent)?;
        }
        // create_dir_all will create the directory if it doesn't exist,
        // or do nothing if it already exists as a directory
        fs::create_dir_all(parent)?;
    }
    
    // Sync guaranteed: Create file, write content, and flush to disk
    let mut file = File::create(&full_path)?;
    file.write_all(s.as_bytes())?;
    file.sync_all()?;  // Forces flush to disk (not just OS buffers)
    
    Ok(())
}

// A simple implementation of `% touch path` (ignores existing files)
pub fn add_file(path: &str) -> io::Result<()> {
    let full_path = get_full_path(path);
    // Ensure parent directory exists
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent)?;
    }
    match OpenOptions::new().create(true).write(true).open(&full_path) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub(crate) fn get_full_path(path: &str) -> PathBuf {
    let base = PROJECT_ROOT
        .get()
        .map(|root| format!("{}/dxn-files/{}", root, FILES_SUBDIR))
        .unwrap_or_else(|| ROOT_FILE_PATH_FALLBACK.to_string());
    PathBuf::from(&base).join(Path::new(path))
}

pub fn add_dir(path: &str) -> io::Result<()> { 
    let mut full_path = get_full_path(path);
    fs::create_dir_all(full_path)?;
    Ok(())
}

#[cfg(test)]
#[path = "manager.test.rs"]
mod tests;
