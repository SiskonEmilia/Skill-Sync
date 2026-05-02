use crate::platform;
use crate::repo::home_dir;
use crate::types::{Cli, SkillEntry};

pub fn run() -> Result<(), String> {
    let home = home_dir()?;

    let mut entries: Vec<SkillEntry> = Vec::new();

    for cli in &[Cli::Claude, Cli::OpenCode] {
        let local_root = cli.local_skill_root(&home);
        if !local_root.is_dir() {
            continue;
        }

        let dirs = std::fs::read_dir(&local_root)
            .map_err(|e| format!("cannot read '{}': {e}", local_root.display()))?;

        for dir in dirs {
            let dir = dir.map_err(|e| format!("read_dir entry error: {e}"))?;
            let metadata = dir.path().metadata();
            if !metadata.map(|m| m.is_dir()).unwrap_or(false) {
                continue;
            }
            let name = dir.file_name().to_string_lossy().to_string();

            let is_link = if cfg!(windows) {
                platform::is_junction(&dir.path())
            } else {
                dir.path().is_symlink()
            };

            let target = if is_link {
                platform::read_link(&dir.path())
            } else {
                None
            };

            entries.push(SkillEntry {
                name,
                cli: *cli,
                is_junction: is_link,
                target: target.map(std::path::PathBuf::from),
            });
        }
    }

    entries.sort_by(|a, b| {
        a.cli
            .dir_name()
            .cmp(b.cli.dir_name())
            .then(a.name.cmp(&b.name))
    });

    if entries.is_empty() {
        println!("No skills found.");
        return Ok(());
    }

    println!("{:<24} {:<12} {:<10} Target", "Name", "CLI", "Link");
    println!("{}", "-".repeat(80));

    for e in &entries {
        let link_type = if e.is_junction {
            if cfg!(windows) {
                "junction"
            } else {
                "symlink"
            }
        } else {
            "local"
        };

        let target = match &e.target {
            Some(t) => t.display().to_string(),
            None => "-".to_string(),
        };

        println!(
            "{:<24} {:<12} {:<10} {}",
            e.name,
            e.cli.to_string(),
            link_type,
            target
        );
    }

    Ok(())
}
