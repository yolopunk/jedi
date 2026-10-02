// Slash palette = agent skills (SKILL.md instruction packages).
// Typing '/' in the chat input lists enabled skills; sending '/name ...'
// loads the SKILL.md body into the run context (see stores/aiChat.ts
// resolveSlashSkill) — the same semantics as Claude Code's /skill.

import { skillRegistry } from '@/skills/registry'

export interface SlashCommand {
  /** with leading '/', e.g. '/hosts-troubleshoot' */
  name: string
  description: string
  icon: string
}

/** 当前已启用技能的斜杠命令清单（调用方需自行建立响应式依赖） */
export function listSkillCommands(): SlashCommand[] {
  return skillRegistry.listEnabled().map(s => ({
    name: `/${s.name}`,
    description: s.description,
    icon: s.icon,
  }))
}
