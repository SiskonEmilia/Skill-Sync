use std::path::{Path, PathBuf};

use crate::repo::home_dir;

pub struct RepoEntry {
    pub path: PathBuf,
    pub is_default: bool,
}

pub fn config_path() -> Result<PathBuf, String> {
    let home = home_dir()?;
    Ok(home.join(".config").join("skill-sync").join("repos"))
}

pub fn load_entries() -> Result<Vec<RepoEntry>, String> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read config '{}': {e}", path.display()))?;

    let entries = content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            if let Some(p) = l.strip_prefix("* ") {
                RepoEntry {
                    path: PathBuf::from(p),
                    is_default: true,
                }
            } else {
                RepoEntry {
                    path: PathBuf::from(l),
                    is_default: false,
                }
            }
        })
        .collect();

    Ok(entries)
}

pub fn load_repos() -> Result<Vec<PathBuf>, String> {
    Ok(load_entries()?.into_iter().map(|e| e.path).collect())
}

pub fn default_repo() -> Result<Option<PathBuf>, String> {
    let entries = load_entries()?;
    Ok(entries.into_iter().find(|e| e.is_default).map(|e| e.path))
}

fn save_entries(entries: &[RepoEntry]) -> Result<(), String> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create config dir '{}': {e}", parent.display()))?;
    }

    let content: String = entries
        .iter()
        .map(|e| {
            if e.is_default {
                format!("* {}", e.path.display())
            } else {
                e.path.display().to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    std::fs::write(&path, content + "\n")
        .map_err(|e| format!("cannot write config '{}': {e}", path.display()))?;

    Ok(())
}

pub fn save_repos(repos: &[PathBuf]) -> Result<(), String> {
    let existing = load_entries().unwrap_or_default();
    let entries: Vec<RepoEntry> = repos
        .iter()
        .map(|p| {
            let is_default = existing.iter().any(|e| e.is_default && e.path == *p);
            RepoEntry {
                path: p.clone(),
                is_default,
            }
        })
        .collect();
    save_entries(&entries)
}

pub fn add_repo(path: &Path) -> Result<(), String> {
    let canonical = std::fs::canonicalize(path)
        .map_err(|e| format!("cannot resolve path '{}': {e}", path.display()))?;

    let mut entries = load_entries()?;
    if entries.iter().any(|e| {
        std::fs::canonicalize(&e.path)
            .map(|c| c == canonical)
            .unwrap_or(false)
    }) {
        eprintln!("Already registered: '{}'", canonical.display());
        return Ok(());
    }

    let is_first = entries.is_empty();
    entries.push(RepoEntry {
        path: canonical.clone(),
        is_default: is_first,
    });
    save_entries(&entries)?;
    eprintln!("Added repo: '{}'", canonical.display());
    if is_first {
        eprintln!("  (set as default)");
    }
    Ok(())
}

pub fn remove_repo(path: &Path) -> Result<(), String> {
    let canonical = std::fs::canonicalize(path)
        .map_err(|e| format!("cannot resolve path '{}': {e}", path.display()))?;

    let mut entries = load_entries()?;
    let before = entries.len();
    entries.retain(|e| {
        std::fs::canonicalize(&e.path)
            .map(|c| c != canonical)
            .unwrap_or(true)
    });

    if entries.len() == before {
        return Err(format!("'{}' is not registered", canonical.display()));
    }

    save_entries(&entries)?;
    eprintln!("Removed repo: '{}'", canonical.display());
    Ok(())
}

pub fn set_default(path: &Path) -> Result<(), String> {
    let canonical = std::fs::canonicalize(path)
        .map_err(|e| format!("cannot resolve path '{}': {e}", path.display()))?;

    let mut entries = load_entries()?;
    let mut found = false;
    for e in &mut entries {
        let matches = std::fs::canonicalize(&e.path)
            .map(|c| c == canonical)
            .unwrap_or(false);
        if matches {
            e.is_default = true;
            found = true;
        } else {
            e.is_default = false;
        }
    }

    if !found {
        return Err(format!(
            "'{}' is not registered — add it first with `sync repo add`",
            canonical.display()
        ));
    }

    save_entries(&entries)?;
    eprintln!("Default repo set to: '{}'", canonical.display());
    Ok(())
}
