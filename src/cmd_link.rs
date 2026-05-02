use std::path::Path;

use crate::platform;
use crate::repo::{home_dir, Repo};
use crate::types::Cli;

pub fn run(name: &str, cli: Cli, repo_override: Option<std::path::PathBuf>) -> Result<(), String> {
    let repos = Repo::all(repo_override)?;
    let home = home_dir()?;

    let target = repos
        .iter()
        .map(|r| r.skill_dir(name, cli))
        .find(|d| d.join("SKILL.md").exists())
        .ok_or_else(|| {
            format!(
                "skill '{}' not found for {} in any registered repo",
                name, cli
            )
        })?;

    let link = cli.local_skill_root(&home).join(name);

    let is_link = if cfg!(windows) {
        platform::is_junction(&link)
    } else {
        Path::is_symlink(&link)
    };

    if link.join("SKILL.md").exists() && !is_link {
        let local_content = std::fs::read_to_string(link.join("SKILL.md")).unwrap_or_default();
        let repo_content = std::fs::read_to_string(target.join("SKILL.md")).unwrap_or_default();
        if local_content != repo_content {
            return Err(format!(
                "'{}' is a local skill with unsaved changes — copy it to the sync repo first, then re-run sync link",
                link.display()
            ));
        }
    }

    if let Some(parent) = link.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create parent dir '{}': {e}", parent.display()))?;
    }

    platform::create_link(&target, &link)?;

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
