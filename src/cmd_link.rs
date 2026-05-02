use std::path::Path;

use crate::platform;
use crate::repo::{home_dir, Repo};
use crate::types::Cli;

pub fn run(name: &str, cli: Cli, repo_override: Option<std::path::PathBuf>) -> Result<(), String> {
    let repo = Repo::detect(repo_override)?;
    let home = home_dir()?;

    let target = repo.skill_dir(name, cli);
    if !target.join("SKILL.md").exists() {
        return Err(format!(
            "skill '{}' not found in repo at '{}'",
            name,
            target.display()
        ));
    }

    let link = cli.local_skill_root(&home).join(name);

    if let Some(parent) = link.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create parent dir '{}': {e}", parent.display()))?;
    }

    platform::create_link(&target, &link)?;

    // Verify
    let verify_path = link.join("SKILL.md");
    verify_path.metadata().map_err(|e| {
        format!(
            "junction verification failed: cannot read '{}': {e}",
            verify_path.display()
        )
    })?;

    let link_type = if cfg!(windows) { "junction" } else { "symlink" };
    eprintln!(
        "Linked {}: '{}' -> '{}'",
        link_type,
        link.display(),
        target.display()
    );
    Ok(())
}

pub fn unlink(name: &str, cli: Cli) -> Result<(), String> {
    let home = home_dir()?;
    let link = cli.local_skill_root(&home).join(name);

    if !link.exists() {
        eprintln!("'{}' does not exist — nothing to unlink", link.display());
        return Ok(());
    }

    let is_link = if cfg!(windows) {
        platform::is_junction(&link)
    } else {
        Path::is_symlink(&link)
    };

    if !is_link {
        return Err(format!(
            "'{}' is not a {} — refusing to unlink",
            link.display(),
            if cfg!(windows) { "junction" } else { "symlink" }
        ));
    }

    platform::remove_link(&link)?;
    eprintln!("Unlinked '{}'", link.display());
    Ok(())
}
