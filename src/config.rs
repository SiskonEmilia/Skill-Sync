use std::path::{Path, PathBuf};

use crate::repo::home_dir;

pub fn config_path() -> Result<PathBuf, String> {
    let home = home_dir()?;
    Ok(home.join(".config").join("skill-sync").join("repos"))
}

pub fn load_repos() -> Result<Vec<PathBuf>, String> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read config '{}': {e}", path.display()))?;

    let repos: Vec<PathBuf> = content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(PathBuf::from)
        .collect();

    Ok(repos)
}

pub fn save_repos(repos: &[PathBuf]) -> Result<(), String> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create config dir '{}': {e}", parent.display()))?;
    }

    let content: String = repos
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join("\n");

    std::fs::write(&path, content + "\n")
        .map_err(|e| format!("cannot write config '{}': {e}", path.display()))?;

    Ok(())
}

pub fn add_repo(path: &Path) -> Result<(), String> {
    let canonical = std::fs::canonicalize(path)
        .map_err(|e| format!("cannot resolve path '{}': {e}", path.display()))?;

    let mut repos = load_repos()?;
    if repos.iter().any(|r| {
        std::fs::canonicalize(r)
            .map(|c| c == canonical)
            .unwrap_or(false)
    }) {
        eprintln!("Already registered: '{}'", canonical.display());
        return Ok(());
    }

    repos.push(canonical.clone());
    save_repos(&repos)?;
    eprintln!("Added repo: '{}'", canonical.display());
    Ok(())
}

pub fn remove_repo(path: &Path) -> Result<(), String> {
    let canonical = std::fs::canonicalize(path)
        .map_err(|e| format!("cannot resolve path '{}': {e}", path.display()))?;

    let mut repos = load_repos()?;
    let before = repos.len();
    repos.retain(|r| {
        std::fs::canonicalize(r)
            .map(|c| c != canonical)
            .unwrap_or(true)
    });

    if repos.len() == before {
        return Err(format!("'{}' is not registered", canonical.display()));
    }

    save_repos(&repos)?;
    eprintln!("Removed repo: '{}'", canonical.display());
    Ok(())
}
