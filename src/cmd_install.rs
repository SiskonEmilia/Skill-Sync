use crate::cmd_link;
use crate::repo::{home_dir, Repo};
use crate::types::Cli;

pub fn run(repo_override: Option<std::path::PathBuf>) -> Result<(), String> {
    let repos = Repo::all(repo_override.clone())?;
    let home = home_dir()?;

    let mut ok = 0;
    let mut skipped = 0;

    for repo in &repos {
        for cli in &[Cli::Claude, Cli::OpenCode] {
            let repo_cli_dir = repo.root.join(cli.dir_name());
            if !repo_cli_dir.is_dir() {
                continue;
            }

            let entries = std::fs::read_dir(&repo_cli_dir)
                .map_err(|e| format!("cannot read '{}': {e}", repo_cli_dir.display()))?;

            for entry in entries {
                let entry = entry.map_err(|e| format!("read_dir entry error: {e}"))?;
                if !entry.path().metadata().map(|m| m.is_dir()).unwrap_or(false) {
                    continue;
                }

                let name = entry.file_name();
                let name = name.to_string_lossy();

                let skiml = repo_cli_dir.join(&*name).join("SKILL.md");
                if !skiml.exists() {
                    continue;
                }

                let local_skill = cli.local_skill_root(&home).join(&*name);

                let is_link = if cfg!(windows) {
                    crate::platform::is_junction(&local_skill)
                } else {
                    local_skill.is_symlink()
                };

                if is_link {
                    let target = if cfg!(windows) {
                        crate::platform::read_link(&local_skill)
                    } else {
                        std::fs::read_link(&local_skill)
                            .ok()
                            .map(|p| p.display().to_string())
                    };

                    match target {
                        Some(t) if t == entry.path().display().to_string() => {
                            skipped += 1;
                            continue;
                        }
                        _ => {
                            eprintln!(
                                "Warning: '{}' points to wrong target — re-linking",
                                local_skill.display()
                            );
                            crate::platform::remove_link(&local_skill)?;
                        }
                    }
                } else if local_skill.exists() {
                    eprintln!(
                        "Warning: '{}' is a real directory, not a link — skipping",
                        local_skill.display()
                    );
                    skipped += 1;
                    continue;
                }

                cmd_link::run(&name, *cli, Some(repo.root.clone()))?;
                ok += 1;
            }
        }
    }

    eprintln!("Done: {} linked, {} skipped", ok, skipped);
    Ok(())
}
