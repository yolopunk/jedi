---
name: Install gstack
description: Install gstack skill suite and configure the Jedi project to use it
type: project
---

# Install gstack Design

## Purpose
Install the gstack skill suite to enhance Claude Code's capabilities for the Jedi project.

## Steps

### 1. Clone gstack repository
- Create `~/.claude/skills` directory if it doesn't exist
- Clone `https://github.com/garrytan/gstack.git` into `~/.claude/skills/gstack`

### 2. Run setup script
- Execute `~/.claude/skills/gstack/setup` to complete installation

### 3. Update CLAUDE.md
Add the following section at the end of `/Users/cynosure/workspace/github/jedi/CLAUDE.md`:

```
## gstack

This project uses the gstack skill suite for enhanced development workflows.

- **Web Browsing**: Use the `/browse` skill from gstack for all web browsing
- **Do NOT use**: `mcp__claude-in-chrome__*` tools

### Available gstack Skills
- `/office-hours`
- `/plan-ceo-review`
- `/plan-eng-review`
- `/plan-design-review`
- `/design-consultation`
- `/review`
- `/ship`
- `/browse`
- `/qa`
- `/qa-only`
- `/design-review`
- `/setup-browser-cookies`
- `/retro`
- `/investigate`
- `/document-release`
- `/codex`
- `/careful`
- `/freeze`
- `/guard`
- `/unfreeze`
- `/gstack-upgrade`
```

### 4. Ask user about project-level gstack
After completing installation, ask the user if they also want to add gstack to the current project so teammates get it.

## Success Criteria
- gstack repository cloned to `~/.claude/skills/gstack`
- Setup script executed successfully
- CLAUDE.md updated with gstack section
- User consulted about project-level gstack configuration
