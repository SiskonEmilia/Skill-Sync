---
name: skill-sync
description: Multi-device skill synchronization via GitHub. Use when the user wants to push, pull, or manage shared agent skills across devices.
license: MIT
compatibility: opencode
metadata:
  audience: all
  instructions: |
    This file is the OpenCode-adapted version generated from src/skill.md.
    Claude-specific frontmatter fields and syntax have been stripped; see
    claude/skill-sync/SKILL.md for the Claude Code equivalent.
    For modification, edit src/skill.md (ground truth) and re-run adaptation.
---

# Skill Sync

Synchronize agent skills across multiple devices using a GitHub repository.

## Architecture

```
~/skill-sync/                              # GitHub sync repo (local clone)
├── claude/<skill>/SKILL.md                # Claude Code skills
├── opencode/<skill>/SKILL.md              # OpenCode skills
├── sync                                   # Compiled binary
└── install.ps1 / install.sh               # Bootstrap scripts

~/.config/opencode/skills/<skill>/ --junction--> ~/skill-sync/opencode/<skill>/
~/.claude/skills/<skill>/ --junction--> ~/skill-sync/claude/<skill>/
```

## Operations

### Bootstrap (new device)

1. Run `git clone <sync-repo-url> ~/skill-sync` and consider its output below
2. Run `./install.ps1` (Windows) or `./install.sh` (Linux/macOS)
3. The install script compiles the binary and runs `sync install` which creates junctions

### Push a skill

1. The build agent reads this skill, determines if user has a new or modified skill
2. Copy local skill directory into `~/skill-sync/<cli>/<skill-name>/`
3. Run `sync link <skill-name> <cli>` to replace local dir with junction
4. Run `git -C ~/skill-sync add`, `git -C ~/skill-sync commit`, `git -C ~/skill-sync push` and consider its output below

### Pull skills from repo

1. Run `git -C ~/skill-sync pull` and consider its output below
2. Run `sync install` to re-establish any missing junctions
3. Run `sync status` and consider its output below to verify all junctions are valid

### Remove a skill

1. Run `sync unlink <skill-name> <cli>` to remove the junction
2. Delete the skill directory from the sync repo
3. Run `git commit` and `git push`

## Commands reference

```
sync install              Scan the repo and create junctions for all skills
sync link <name> <cli>    Create junction linking local skill dir to repo
sync unlink <name> <cli>  Remove a junction at the local skill dir
sync status               List all local skills and their sync status
```

## Prerequisites

- **Windows**: Developer Mode enabled (required for `mklink /J`)
- **Linux/macOS**: Standard symlink support (built-in)
- Git with `core.symlinks=false` (default on Windows — junctions are not symlinks so this is fine)

## Compatibility adapter rules

### Opening a skill from the sync repo

When the user asks to push a skill or sync skills:
1. Read the current skill content from the local user skill directory
2. Identify which CLI it belongs to (check if it's in `~/.config/opencode/skills/` or `~/.claude/skills/`)
3. Identify the skill name (directory name = skill name)

### Porting Claude Code skill → OpenCode

These rules describe how to strip Claude-specific features from a skill to create an OpenCode-compatible version.

**Frontmatter:**
- Keep: `name`, `description`
- Strip: `allowed-tools`, `context`, `agent`, `hooks`, `paths`, `shell`, `disable-model-invocation`, `user-invocable`, `effort`, `model`, `argument-hint`, `arguments`, `when_to_use`
- Add: `license: MIT`, `metadata: { source: claude }`

**Content transformations:**
- `` !`<cmd>` `` inline shell injection: Replace with `Run `<cmd>` and consider its output below.`
- ` ```! ` fenced shell injection: Replace with `Run the following command and apply its output:` followed by a fenced code block
- `${CLAUDE_SKILL_DIR}`: Replace with relative path instruction to the skill directory
- `${CLAUDE_SESSION_ID}`: Replace with instruction to generate a unique ID
- `Bash(<tool>)`: Replace with the raw command
- `context: fork` usage (run in subagent): Replace with instruction to use a sub-agent or plan mode equivalent

### Porting OpenCode skill → Claude Code

**Frontmatter:**
- Keep: `name`, `description`, `metadata`
- Strip: `license`, `compatibility`
- Add: `disable-model-invocation: true`

**Content transformations:**
- Agent references: OpenCode uses `plan`, `build`. Claude Code uses `Plan`, `Explore`, `general-purpose`.
- Verify skill name conforms to Claude Code pattern (lowercase, hyphens, 1-64 chars)

### Cross-CLI skill maintenance

When a skill exists in both `opencode/` and `claude/` directories:
1. The universal skill source file (`src/skill.md` for this skill) is the ground truth
2. Apply the compatibility adapter rules above to generate both adapted versions
3. Replace both adapted versions with the regenerated content
4. Run `sync install` to ensure junctions point to the right files

## Conflict resolution

When the same skill has been modified on two devices:

1. Run `git -C ~/skill-sync fetch` and consider its output below; check for merge conflicts
2. If conflicts exist, compare both versions
3. Prefer the version with more substantive changes (larger diff, newer content)
4. If uncertain, present the diff to the user and ask which to keep
5. After resolving, run `git add`, `git commit`, `git push`
6. On the other device: run `git pull` (no junction changes needed since junction targets are file-level)

## Notes

- **Never** run `sync link` on a skill that is already managed by a separate git repo
- **Always** verify a skill was successfully junctioned by reading a file through the junction path
- The `sync` binary auto-detects the repo root from its own location
- Junctions on Windows are not the same as git symlinks — `core.symlinks=true` is NOT required
