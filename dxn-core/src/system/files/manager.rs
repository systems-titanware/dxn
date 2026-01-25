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

const ROOT_FILE_PATH: &str = "../dxn-files/_files";

// A simple implementation of `% cat path`
pub fn read_file(path: &str) -> io::Result<String> {
    let mut full_path = get_full_path(path);
    let mut f = File::open(full_path.as_os_str())?;
    let mut s = String::new();
    match f.read_to_string(&mut s) {
        Ok(_) => Ok(s),
        Err(e) => Err(e),
    }
}

// A simple implementation of `% echo s > path`
pub fn add_content(str: &str, path: &str) -> io::Result<()> {
    let mut full_path = get_full_path(path);
    // Open a file with append option.
    // .append(true) ensures new data is added to the end of the file.
    // .create(true) will create the file if it doesn't exist.
    let mut file = OpenOptions::new()
        .append(true)
        .create(true) // Create the file if it doesn't exist
        .open(full_path)?; // Use `?` for concise error propagation

    // Write content to the file.
    // .write_all() takes a byte slice.
    file.write_all(str.as_bytes())
}

// A simple implementation of `% echo s > path`
pub fn add_file_content(s: &str, path: &str) -> io::Result<()> {
    let full_path = get_full_path(path);
    // Ensure parent directory exists
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent)?;
    }
    // Use fs::write which handles file creation and writing atomically
    fs::write(&full_path, s.as_bytes())
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
    let mut full_path = PathBuf::new();
    full_path.push(ROOT_FILE_PATH);
    full_path.push(Path::new(path));
    full_path
}

pub fn add_dir(path: &str) -> io::Result<()> { 
    let mut full_path = get_full_path(path);
    fs::create_dir_all(full_path)?;
    Ok(())
}

#[cfg(test)]
#[path = "manager.test.rs"]
mod tests;
