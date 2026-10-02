// Skills: SKILL.md instruction packages (see types.ts). Tools live in
// src/agent/tools/ — this barrel only exports the instruction-skill layer.

export { BUILTIN_SKILLS } from './builtin'
export { parseSkillDoc } from './frontmatter'
export { invalidateSkill, loadSkill } from './loader'
export { SkillRegistry, skillRegistry } from './registry'
export type { SkillDetail, SkillManifest, SkillSource } from './types'
