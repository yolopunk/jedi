# Install gstack Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps uses checkbox (`- [ ]`) syntax for tracking.

**Goal:** Install gstack skill suite and configure the Jedi project to use it.

**Architecture:** Clone gstack repo, run setup script, update CLAUDE.md with gstack configuration.

**Tech Stack:** Git, Bash

---

## File Structure

| File | Operation | Purpose |
|------|-----------|---------|
| `~/.claude/skills/gstack` | Create (git clone) | gstack skill suite |
| `/Users/cynosure/workspace/github/jedi/CLAUDE.md` | Modify | Add gstack configuration section |

---

### Task 1: Clone gstack repository

**Files:**
- Create: `~/.claude/skills/gstack` (via git clone)

- [ ] **Step 1: Create skills directory**

Run:
```bash
mkdir -p ~/.claude/skills
```

- [ ] **Step 2: Clone gstack repository**

Run:
```bash
cd ~/.claude/skills && git clone https://github.com/garrytan/gstack.git
```

Expected: Repository cloned successfully into `~/.claude/skills/gstack`

- [ ] **Step 3: Verify clone**

Run:
```bash
ls -la ~/.claude/skills/gstack
```

Expected: Directory contents visible including `setup` script

---

### Task 2: Run gstack setup script

**Files:**
- Execute: `~/.claude/skills/gstack/setup`

- [ ] **Step 1: Run setup script**

Run:
```bash
cd ~/.claude/skills/gstack && ./setup
```

Expected: Setup script completes successfully

---

### Task 3: Update CLAUDE.md with gstack section

**Files:**
- Modify: `/Users/cynosure/workspace/github/jedi/CLAUDE.md`

- [ ] **Step 1: Read current CLAUDE.md**

Read: `/Users/cynosure/workspace/github/jedi/CLAUDE.md`

- [ ] **Step 2: Append gstack section**

Add the following at the end of the file:

```markdown
---

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

- [ ] **Step 3: Commit the change**

Run:
```bash
cd /Users/cynosure/workspace/github/jedi && git add CLAUDE.md && git commit -m "docs: add gstack configuration to CLAUDE.md

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

### Task 4: Ask user about project-level gstack

**Files:** None

- [ ] **Step 1: Consult user**

Ask: "Do you also want to add gstack to the current project so teammates get it?"
