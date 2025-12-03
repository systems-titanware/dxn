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

const ROOT_FILE_PATH: &str = "./_files";

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
pub fn add_content(s: &str, path: &str) -> io::Result<()> {
    let mut full_path = get_full_path(path);
    let mut f = File::create(full_path)?;
    f.write_all(s.as_bytes())
}

// A simple implementation of `% touch path` (ignores existing files)
pub fn add_file(path: &str) -> io::Result<()> {
    let mut full_path = get_full_path(path);
    match OpenOptions::new().create(true).write(true).open(full_path) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

fn get_full_path(path: &str) -> PathBuf {
    let mut full_path = PathBuf::new();
    full_path.push(ROOT_FILE_PATH);
    full_path.push(Path::new(path));
    full_path
}

pub fn add_dir(path: &str) -> io::Result<()> { 
    let mut full_path = get_full_path(path);
    println!("Creating path : {:?}", full_path); // Output: Path: "base_dir/subdir/file.txt" (or similar, depending on OS)
    fs::create_dir_all(full_path)?;
    Ok(())
}
