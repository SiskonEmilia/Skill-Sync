use std::path::{Path, PathBuf};

pub struct SkillEntry {
    pub name: String,
    pub cli: Cli,
    pub is_junction: bool,
    pub target: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cli {
    Claude,
    OpenCode,
}

impl Cli {
    pub fn dir_name(&self) -> &str {
        match self {
            Cli::Claude => "claude",
            Cli::OpenCode => "opencode",
        }
    }

    pub fn from_dir_name(s: &str) -> Option<Self> {
        match s {
            "claude" => Some(Cli::Claude),
            "opencode" => Some(Cli::OpenCode),
            _ => None,
        }
    }

    pub fn local_skill_root(&self, home: &Path) -> PathBuf {
        match self {
            Cli::Claude => home.join(".claude").join("skills"),
            Cli::OpenCode => home.join(".config").join("opencode").join("skills"),
        }
    }
}

impl std::fmt::Display for Cli {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.dir_name())
    }
}
