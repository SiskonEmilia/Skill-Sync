use std::path::{Path, PathBuf};
use tempfile::TempDir;

use skill_sync::platform;
use skill_sync::repo::{home_dir, Repo};
use skill_sync::types::Cli;

fn setup_repo() -> (TempDir, Repo) {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("claude").join("test-skill")).unwrap();
    std::fs::write(
        dir.path()
            .join("claude")
            .join("test-skill")
            .join("SKILL.md"),
        "---\nname: test-skill\ndescription: A test skill\n---\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.path().join("opencode").join("test-skill")).unwrap();
    std::fs::write(
        dir.path()
            .join("opencode")
            .join("test-skill")
            .join("SKILL.md"),
        "---\nname: test-skill\ndescription: A test skill\n---\n",
    )
    .unwrap();
    let repo = Repo::detect(Some(dir.path().to_path_buf())).unwrap();
    (dir, repo)
}

#[test]
fn test_repo_detect() {
    let (_dir, repo) = setup_repo();
    assert!(repo.root.join("claude").is_dir());
    assert!(repo.root.join("opencode").is_dir());
}

#[test]
fn test_skill_dir_path() {
    let (_dir, repo) = setup_repo();
    let path = repo.skill_dir("test-skill", Cli::Claude);
    assert_eq!(path, repo.root.join("claude").join("test-skill"));
}

#[test]
fn test_cli_parse() {
    assert_eq!(Cli::from_dir_name("claude"), Some(Cli::Claude));
    assert_eq!(Cli::from_dir_name("opencode"), Some(Cli::OpenCode));
    assert_eq!(Cli::from_dir_name("invalid"), None);
}

#[test]
fn test_cli_dir_name() {
    assert_eq!(Cli::Claude.dir_name(), "claude");
    assert_eq!(Cli::OpenCode.dir_name(), "opencode");
}

#[test]
fn test_cli_local_skill_root() {
    let home = Path::new("/home/user");
    assert_eq!(
        Cli::Claude.local_skill_root(home),
        PathBuf::from("/home/user/.claude/skills")
    );
    assert_eq!(
        Cli::OpenCode.local_skill_root(home),
        PathBuf::from("/home/user/.config/opencode/skills")
    );
}

#[test]
fn test_home_dir() {
    let home = home_dir().unwrap();
    assert!(home.is_dir());
}

#[test]
fn test_link_and_unlink() {
    let (_dir, repo) = setup_repo();
    let home = home_dir().unwrap();

    let name = "skill-sync-test-link";
    let target = repo.skill_dir("test-skill", Cli::OpenCode);
    let local = Cli::OpenCode.local_skill_root(&home).join(name);

    // Clean up from previous runs
    if platform::is_junction(&local) {
        let _ = platform::remove_link(&local);
    } else if local.exists() {
        let _ = std::fs::remove_dir_all(&local);
    }

    // Create link
    if let Some(parent) = local.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    platform::create_link(&target, &local).unwrap();
    assert!(local.exists());
    assert!(platform::is_junction(&local));

    // Read through junction
    let content = std::fs::read_to_string(local.join("SKILL.md")).unwrap();
    assert!(content.contains("test-skill"));

    // Remove link
    platform::remove_link(&local).unwrap();
    assert!(!local.exists());
}

#[test]
fn test_status_command_no_crash() {
    let home = home_dir().unwrap();
    for cli in &[Cli::Claude, Cli::OpenCode] {
        let root = cli.local_skill_root(&home);
        if root.is_dir() {
            skill_sync::cmd_status::run().unwrap();
            return;
        }
    }
    skill_sync::cmd_status::run().unwrap();
}

#[test]
fn test_link_refuses_unsaved_local_skill() {
    use tempfile::TempDir;

    let dir = TempDir::new().unwrap();
    let name = "skill-sync-test-refuse";
    std::fs::create_dir_all(dir.path().join("claude").join(name)).unwrap();
    std::fs::write(
        dir.path().join("claude").join(name).join("SKILL.md"),
        "---\nname: skill-sync-test-refuse\ndescription: a\n---\nrepo version",
    )
    .unwrap();
    std::fs::create_dir_all(dir.path().join("opencode")).unwrap();

    let home = home_dir().unwrap();
    let local = Cli::Claude.local_skill_root(&home).join(name);
    let _ = platform::remove_link(&local);
    let _ = std::fs::remove_dir_all(&local);
    std::fs::create_dir_all(&local).unwrap();
    std::fs::write(local.join("SKILL.md"), "---\nname: skill-sync-test-refuse\ndescription: b\n---\nlocal version")
        .unwrap();

    let result = skill_sync::cmd_link::run(name, Cli::Claude, Some(dir.path().to_path_buf()));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("unsaved changes"));

    std::fs::remove_dir_all(&local).unwrap();
}

#[test]
fn test_link_allows_matching_content() {
    use tempfile::TempDir;

    let dir = TempDir::new().unwrap();
    let name = "skill-sync-test-match";
    std::fs::create_dir_all(dir.path().join("claude").join(name)).unwrap();
    let content = "---\nname: skill-sync-test-match\ndescription: a\n---\n";
    std::fs::write(
        dir.path().join("claude").join(name).join("SKILL.md"),
        content,
    )
    .unwrap();
    std::fs::create_dir_all(dir.path().join("opencode")).unwrap();

    let home = home_dir().unwrap();
    let local = Cli::Claude.local_skill_root(&home).join(name);
    let _ = platform::remove_link(&local);
    let _ = std::fs::remove_dir_all(&local);
    std::fs::create_dir_all(&local).unwrap();
    std::fs::write(local.join("SKILL.md"), content).unwrap();

    skill_sync::cmd_link::run(name, Cli::Claude, Some(dir.path().to_path_buf())).unwrap();
    assert!(platform::is_junction(&local));

    let _ = platform::remove_link(&local);
    let _ = std::fs::remove_dir_all(&local);
}
