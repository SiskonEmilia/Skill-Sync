use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use skill_sync::cmd_install;
use skill_sync::cmd_link;
use skill_sync::cmd_repo;
use skill_sync::cmd_status;
use skill_sync::types::Cli;

#[derive(Parser)]
#[command(name = "sync", about = "Multi-device skill synchronization tool")]
struct CliApp {
    #[command(flatten)]
    global: GlobalArgs,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Args)]
struct GlobalArgs {
    /// Path to a skill-sync repo. Auto-detected from config or binary location if omitted.
    #[arg(long, global = true)]
    repo: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan all registered repos and create junctions/symlinks for all skills
    Install,

    /// Create a junction/symlink linking a local skill dir to the repo
    Link {
        /// Skill name
        name: String,
        /// Target CLI
        cli: String,
    },

    /// Remove a junction/symlink from the local skill dir
    Unlink {
        /// Skill name
        name: String,
        /// Target CLI
        cli: String,
    },

    /// List all local skills and their sync status
    Status,

    /// Manage registered skill repos
    Repo {
        #[command(subcommand)]
        action: RepoAction,
    },
}

#[derive(Subcommand)]
enum RepoAction {
    /// Register a skill repo
    Add {
        /// Path to the repo directory
        path: PathBuf,
    },
    /// Unregister a skill repo
    Remove {
        /// Path to the repo directory
        path: PathBuf,
    },
    /// List all registered repos
    List,
    /// Set or show the default repo for new skills
    Default {
        /// Path to set as default. Omit to show current default.
        path: Option<PathBuf>,
    },
}

fn parse_cli(s: &str) -> Result<Cli, String> {
    Cli::from_dir_name(s)
        .ok_or_else(|| format!("unknown CLI '{}' — expected 'claude' or 'opencode'", s))
}

fn main() {
    let app = CliApp::parse();
    let repo_override = app.global.repo;

    let result = match app.command {
        Commands::Install => cmd_install::run(repo_override),
        Commands::Link { name, cli } => {
            let cli = parse_cli(&cli).unwrap_or_else(|e| {
                eprintln!("Error: {e}");
                std::process::exit(1);
            });
            cmd_link::run(&name, cli, repo_override)
        }
        Commands::Unlink { name, cli } => {
            let cli = parse_cli(&cli).unwrap_or_else(|e| {
                eprintln!("Error: {e}");
                std::process::exit(1);
            });
            cmd_link::unlink(&name, cli)
        }
        Commands::Status => cmd_status::run(),
        Commands::Repo { action } => match action {
            RepoAction::Add { path } => cmd_repo::add(path),
            RepoAction::Remove { path } => cmd_repo::remove(path),
            RepoAction::List => cmd_repo::list(),
            RepoAction::Default { path } => match path {
                Some(p) => cmd_repo::set_default(p),
                None => cmd_repo::show_default(),
            },
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
