---
name: skill-sync
description: Multi-device skill synchronization via GitHub. Use when the user wants to push, pull, or manage shared agent skills across devices.
metadata:
  audience: all
  instructions: |
    This is the universal source. claude/skill-sync/SKILL.md and
    opencode/skill-sync/SKILL.md are generated from this file by
    applying the Compatibility adapter rules below. Edit this file
    first, then regenerate the adapted versions.
---

# Skill Sync

Synchronize agent skills across multiple devices using a GitHub repository.

## Architecture

```
~/skill-sync/               # GitHub sync repo (local clone)
├── claude/<skill>/SKILL.md  # Claude Code skills
├── opencode/<skill>/SKILL.md # OpenCode skills
├── sync                     # Compiled binary
└── install.ps1 / install.sh # Bootstrap scripts

~/.claude/skills/<skill>/    --junction--> ~/skill-sync/claude/<skill>/
~/.config/opencode/skills/<skill>/ --junction--> ~/skill-sync/opencode/<skill>/
```

## Operations

### Bootstrap (new device)

1. `git clone <sync-repo-url> ~/skill-sync`
2. Run `./install.ps1` (Windows) or `./install.sh` (Linux/macOS)
3. The install script compiles the binary and runs `sync install` which creates junctions

### Push a skill

1. Agent reads this skill, determines if user has a new or modified skill
2. Copy local skill directory into `~/skill-sync/<cli>/<skill-name>/` (the CLI the skill originates from)
3. Apply the Compatibility adapter rules below to generate the adapted version for the other CLI
4. Copy the adapted version into the other CLI's directory (e.g. `~/skill-sync/opencode/<skill-name>/` if the source was Claude Code)
5. Run `sync link <skill-name> <cli>` to replace local dir with junction — run for both claude and opencode
6. `git add`, `git commit`, `git push` in the sync repo

### Pull skills from repo

1. `git pull` in `~/skill-sync/`
2. Run `sync install` to re-establish any missing junctions
3. Run `sync status` to verify all junctions are valid

### Remove a skill

1. Run `sync unlink <skill-name> <cli>` to remove the junction
2. Delete the skill directory from the sync repo
3. `git commit` and `git push`

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
2. Identify which CLI it belongs to (check if it's in `~/.claude/skills/` or `~/.config/opencode/skills/`)
3. Identify the skill name (directory name = skill name)

### Porting Claude Code skill → OpenCode

**Frontmatter:**
- Keep: `name`, `description`
- Strip: `allowed-tools`, `context`, `agent`, `hooks`, `paths`, `shell`, `disable-model-invocation`, `user-invocable`, `effort`, `model`, `argument-hint`, `arguments`, `when_to_use`
- Add: `license: MIT`, `metadata: { source: claude }`
- If `model` or `effort` was set, add `metadata.requires: { capability: "<model>/<effort>" }` as a hint to the OpenCode agent that this skill was designed for a high-capability context

**Content transformations:**
- `` !\`<cmd>\` `` inline shell injection: Replace with note `Run \`<cmd>\` and consider its output below.`
- ` ```! ` fenced shell injection: Replace with `Run the following command and apply its output:` followed by ` ``` ` fenced code block
- `${CLAUDE_SKILL_DIR}`: Replace with relative path instruction to the skill directory
- `${CLAUDE_SESSION_ID}`: Replace with instruction to generate a unique ID
- `$ARGUMENTS`: Replace with `{{input}}` placeholder and add a note at the top: `This skill expects user-provided input where {{input}} appears.`
- `Bash(<tool>)`: Replace with the raw command
- `context: fork` usage (run in subagent): Replace with instruction to use a sub-agent/plan mode equivalent
- Add compatibility note in metadata

**Tool/agent name mapping:**

Claude Code references in body → OpenCode equivalents:

| Claude Code | OpenCode | Notes |
|-------------|----------|-------|
| `Agent` (tool) | `plan` / sub-agent instruction | OpenCode has no `Agent` tool; rewrite as delegation instruction |
| `Plan`, `Explore`, `general-purpose` (subagent types) | `plan`, `build`, general instruction | Map by capability, not name |
| `Read` | `cat` / file read instruction | Direct tool names may not exist; use natural language |
| `Grep` | `grep` / search instruction | Same |
| `Glob` | `find` / file listing instruction | Same |
| `Edit` | `sed` / edit instruction | Same |
| `WebFetch`, `WebSearch` | `fetch` / web instruction | Verify OpenCode supports web access |
| `Bash` | shell command instruction | Usually 1:1 |
| `NotebookEdit` | notebook instruction | May not be supported |

When a body references tools from `allowed-tools`, audit each reference:
- If OpenCode has an equivalent, map it
- If no equivalent exists, rewrite as a natural-language instruction describing the desired action

**Semantic flags (emit warnings, do not silently drop):**
- If `when_to_use` was set: Add a comment at the top of the OpenCode version: `<!-- Auto-trigger: original skill was model-invocable when: "<when_to_use value>" -->`. OpenCode agents should treat this as a hint for when to suggest the skill.
- If `user-invocable: true` was set with an `argument-hint`: Note the expected invocation pattern at the top so the user knows how to call it.

**Multi-file skill assets:**
- If the skill directory contains files beyond `SKILL.md` (templates, configs, data), verify each referenced path:
  - `${CLAUDE_SKILL_DIR}/foo.json` → rewrite as relative path `./foo.json` with a note: `This file is co-located in the skill directory.`
  - Ensure all auxiliary files are copied to the OpenCode skill directory

### Porting OpenCode skill → Claude Code

**Frontmatter:**
- Keep: `name`, `description`, `metadata`
- Strip: `license`, `compatibility`
- Conditionally add Claude-specific fields:
  - `disable-model-invocation: true` — ONLY if the skill performs side effects (file writes, git operations, network calls). Read-only / advisory skills should remain model-invocable.
  - `allowed-tools` — If the skill body references specific shell commands or actions, list the Claude tools needed (e.g., `Bash`, `Read`, `Grep`)
  - `when_to_use` — If `metadata` contains trigger hints or the skill's description implies automatic activation, convert to a `when_to_use` value
  - `argument-hint` — If the skill expects user input (check for `{{input}}` or similar placeholders), add a hint describing expected arguments
- Verify skill name conforms to Claude Code pattern: `^[a-z0-9][a-z0-9-]{0,63}$`

**Content transformations:**

| OpenCode | Claude Code | Notes |
|----------|-------------|-------|
| `plan` (agent) | `Plan` (subagent_type) | |
| `build` (agent) | `general-purpose` or `Explore` | Match by intent |
| `{{input}}` placeholder | `$ARGUMENTS` | |
| `./foo.json` (co-located file) | `${CLAUDE_SKILL_DIR}/foo.json` | |
| Natural-language tool instructions ("run grep to find...") | Explicit tool reference (`Use the Grep tool`) | Claude Code performs better with explicit tool names |
| `<!-- Auto-trigger: ... -->` comments | `when_to_use` frontmatter field | Restore auto-trigger semantics |

**Validation checks after porting:**
- Name matches `^[a-z0-9][a-z0-9-]{0,63}$`
- No orphan `{{input}}` left unconverted
- All referenced co-located files exist in the Claude skill directory
- If skill uses web access, `WebFetch`/`WebSearch` are in `allowed-tools`

### Cross-CLI skill maintenance

When a skill exists in both `opencode/` and `claude/` directories:
1. The universal skill source file (`src/skill.md` for this skill) is the ground truth
2. Run the compatibility adapter rules above to generate both adapted versions
3. Replace both adapted versions with the regenerated content
4. Run `sync install` to ensure junctions point to the right files

### Known non-portable patterns

Some skill patterns cannot be faithfully converted. When encountered, emit a warning and degrade gracefully:

| Pattern | Direction | Degradation |
|---------|-----------|-------------|
| `context: fork` with return-value dependency | Claude → OpenCode | Loses structured return; rewrite as sequential instructions |
| `hooks` (pre/post execution) | Claude → OpenCode | No equivalent; inline the hook logic into the skill body or drop with warning |
| `paths` scoping | Claude → OpenCode | No equivalent; add a note "This skill is designed for use in directories matching: ..." |
| Real-time streaming (`Monitor` tool) | Claude → OpenCode | Replace with polling instruction |
| OpenCode plugin-specific APIs | OpenCode → Claude | No equivalent; rewrite as Bash/tool calls or mark as unsupported |

## Conflict resolution

When the same skill has been modified on two devices:

1. Run `git fetch` and check for merge conflicts
2. If conflicts exist, compare both versions
3. Prefer the version with more substantive changes (larger diff, newer content)
4. If uncertain, present the diff to the user and ask which to keep
5. After resolving, `git add`, `git commit`, `git push`
6. On the other device: `git pull` (no junction changes needed since junction targets are file-level)

## Notes

- **Never** run `sync link` on a skill that is already managed by a separate git repo
- **Always** verify a skill was successfully junctioned by reading a file through the junction path
- The `sync` binary auto-detects the repo root from its own location
- Junctions on Windows are not the same as git symlinks — `core.symlinks=true` is NOT required
