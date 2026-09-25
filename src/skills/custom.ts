// src/skills/custom.ts
//
// Factory that turns a user-defined markdown skill (CustomSkillDef from
// ~/.jedi/skills/, parsed by the Rust backend) into a registry-ready Skill.
// Custom skills are prompt skills: executing one renders {{placeholder}}
// args into the prompt and returns it as the tool result, and the model
// acts on it with the built-in tools (which keep their own risk gates).

import type { CustomSkillDef, Skill, SkillRisk } from './types'

const PLACEHOLDER_RE = /\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\}\}/g

function riskOf(risk: string): SkillRisk {
  return risk === 'write' || risk === 'system' ? risk : 'read'
}

/** 提取提示词正文中的 {{param}} 占位符名（去重、保持出现顺序） */
export function extractPlaceholders(prompt: string): string[] {
  const names: string[] = []
  for (const match of prompt.matchAll(PLACEHOLDER_RE)) {
    if (!names.includes(match[1])) names.push(match[1])
  }
  return names
}

/** 将 {{param}} 替换为 args 值；缺参时保留占位符原样，便于模型发现并补齐 */
export function renderPrompt(prompt: string, args: Record<string, unknown>): string {
  return prompt.replace(PLACEHOLDER_RE, (raw, name: string) => {
    const value = args[name]
    return value === undefined || value === null ? raw : String(value)
  })
}

/** 把自定义技能定义包装为可注册进 skillRegistry 的 Skill */
export function createCustomSkill(def: CustomSkillDef): Skill {
  const params = extractPlaceholders(def.prompt)
  return {
    id: def.id,
    name: def.name,
    description: def.description,
    icon: def.icon,
    enabled: true,
    autoCallable: true,
    risk: riskOf(def.risk),
    source: 'custom',
    parameters: {
      type: 'object',
      properties: Object.fromEntries(
        params.map(p => [p, { type: 'string' as const, description: p, required: true }])
      ),
    },
    execute: async (args: Record<string, unknown>) => ({
      type: 'prompt',
      prompt: renderPrompt(def.prompt, args ?? {}),
    }),
  }
}
