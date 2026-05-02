# Skill-Sync

Multi-device agent skill synchronization via GitHub. Share Claude Code and OpenCode skills across all your machines.

## Quick Start

```bash
git clone <your-sync-repo-url> ~/skill-sync
cd ~/skill-sync
./install.ps1   # Windows
# or
./install.sh    # Linux / macOS
```

The install script compiles the Rust binary and creates junctions (Windows) or symlinks (Linux/macOS) in your user-level skill directories.

## How It Works

```
~/.claude/skills/<skill>/    ──junction──→  ~/skill-sync/claude/<skill>/
~/.config/opencode/skills/<skill>/ ──junction──→  ~/skill-sync/opencode/<skill>/
```

Skills are stored as real files in your sync repo. Local skill directories are transparent junctions/symlinks pointing into the repo. Push from one device, pull on another — no manual file copying.

## Commands

```
sync install              Scan repo and create links for all skills
sync link <name> <cli>    Link one skill (cli = claude | opencode)
sync unlink <name> <cli>  Remove a skill link
sync status               List all local skills and their link status
```

Use `--repo <path>` to override auto-detection of the sync repo root.

## Prerequisites

- **Windows**: Developer Mode enabled (required for `mklink /J`)
- **Linux / macOS**: Standard symlink support (built-in)
- Rust toolchain (for compilation)

## Project Structure

```
Skill-Sync/
├── claude/                         # Claude Code adapted skills
├── opencode/                       # OpenCode adapted skills
├── src/                            # Rust source
│   └── skill.md                    # skill-sync source (ground truth)
├── tests/                          # Integration tests
├── install.ps1 / install.sh        # Bootstrap scripts
└── docs/internal/                  # Design docs (Chinese)
```

## License

[GPL-3.0](LICENSE)
