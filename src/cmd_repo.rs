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
    let repos = config::load_repos()?;
    if repos.is_empty() {
        println!("No repos registered. Use `sync repo add <path>` to register one.");
        return Ok(());
    }
    for r in &repos {
        let exists = r.is_dir();
        let marker = if exists { " " } else { "!" };
        println!("{} {}", marker, r.display());
    }
    Ok(())
}
