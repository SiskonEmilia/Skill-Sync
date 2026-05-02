use std::path::PathBuf;

use crate::config;

pub fn add(path: PathBuf) -> Result<(), String> {
    if !path.is_dir() {
        return Err(format!("'{}' is not a directory", path.display()));
    }
    config::add_repo(&path)
}

pub fn remove(path: PathBuf) -> Result<(), String> {
    config::remove_repo(&path)
}

pub fn list() -> Result<(), String> {
    let entries = config::load_entries()?;
    if entries.is_empty() {
        println!("No repos registered. Use `sync repo add <path>` to register one.");
        return Ok(());
    }
    for e in &entries {
        let exists = e.path.is_dir();
        let marker = if !exists {
            "!"
        } else if e.is_default {
            "*"
        } else {
            " "
        };
        println!("{} {}", marker, e.path.display());
    }
    Ok(())
}

pub fn set_default(path: PathBuf) -> Result<(), String> {
    config::set_default(&path)
}

pub fn show_default() -> Result<(), String> {
    match config::default_repo()? {
        Some(p) => println!("{}", p.display()),
        None => println!("No default repo set."),
    }
    Ok(())
}
