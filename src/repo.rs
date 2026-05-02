use std::path::{Path, PathBuf};

use crate::config;
use crate::types::Cli;

#[derive(Debug)]
pub struct Repo {
    pub root: PathBuf,
}

fn is_repo_root(dir: &Path) -> bool {
    dir.join("claude").is_dir() || dir.join("opencode").is_dir()
}

impl Repo {
    pub fn detect(repo_override: Option<PathBuf>) -> Result<Self, String> {
        if let Some(p) = repo_override {
            return if is_repo_root(&p) {
                Ok(Repo { root: p })
            } else {
                Err(format!(
                    "not a valid skill-sync repo (need 'claude/' or 'opencode/' in '{}')",
                    p.display()
                ))
            };
        }

        let exe = std::env::current_exe().map_err(|e| format!("cannot get binary path: {e}"))?;
        let mut current = exe
            .parent()
            .ok_or_else(|| "binary has no parent directory".to_string())?
            .to_path_buf();

        while !is_repo_root(&current) {
            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => {
                    return Err(format!(
                        "not a valid skill-sync repo — no 'claude/' or 'opencode/' found in or above '{}'",
                        exe.parent().unwrap_or_else(|| Path::new(".")).display()
                    ));
                }
            }
        }

        Ok(Repo { root: current })
    }

    pub fn all(repo_override: Option<PathBuf>) -> Result<Vec<Self>, String> {
        if let Some(p) = repo_override {
            return Ok(vec![Self::detect(Some(p))?]);
        }

        let configured = config::load_repos()?;
        if configured.is_empty() {
            return Ok(vec![Self::detect(None)?]);
        }

        let mut repos = Vec::new();
        for path in configured {
            if is_repo_root(&path) {
                repos.push(Repo { root: path });
            } else {
                eprintln!(
                    "Warning: '{}' is not a valid skill-sync repo — skipping",
                    path.display()
                );
            }
        }

        if repos.is_empty() {
            return Err("no valid repos found in config".to_string());
        }

        Ok(repos)
    }

    pub fn skill_dir(&self, name: &str, cli: Cli) -> PathBuf {
        self.root.join(cli.dir_name()).join(name)
    }
}

pub fn home_dir() -> Result<PathBuf, String> {
    dirs_fallback()
}

fn dirs_fallback() -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        std::env::var("USERPROFILE")
            .map(PathBuf::from)
            .map_err(|_| "USERPROFILE not set".to_string())
    }
    #[cfg(not(windows))]
    {
        std::env::var("HOME")
            .map(PathBuf::from)
            .map_err(|_| "HOME not set".to_string())
    }
}
