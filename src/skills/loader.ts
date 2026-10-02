// Skill body loader — the runtime half of progressive disclosure.
// Builtin bodies come from the raw-imported SKILL.md packages; user skill
// bodies are fetched from the backend (skills_read) and cached for the
// session. The system prompt only ever carries name+description; this module
// is what the `skill` tool and the /slash invocation call to load the body.

import { readSkill } from '@/api/skills'
import { BUILTIN_SKILLS } from './builtin'

export interface LoadedSkill {
  body: string
  /** bundled files relative to the skill directory */
  files: string[]
  /** absolute skill directory (user skills; empty for builtin) */
  dir: string
}

const cache = new Map<string, LoadedSkill>()

/** 加载技能内容（内置走打包资源，用户技能走后端并缓存） */
export async function loadSkill(name: string): Promise<LoadedSkill> {
  const cached = cache.get(name)
  if (cached) return cached
  const builtin = BUILTIN_SKILLS.find(s => s.name === name)
  if (builtin) {
    const loaded: LoadedSkill = { body: builtin.body, files: [], dir: '' }
    cache.set(name, loaded)
    return loaded
  }
  const detail = await readSkill(name)
  const loaded: LoadedSkill = { body: detail.body, files: detail.files, dir: detail.dir }
  cache.set(name, loaded)
  return loaded
}

/** 技能保存/删除后使其缓存失效 */
export function invalidateSkill(name?: string): void {
  if (name === undefined) cache.clear()
  else cache.delete(name)
}
