// src/skills/types.ts

export interface ParameterSchema {
  type: 'object'
  properties: Record<
    string,
    {
      type: 'string' | 'number' | 'boolean' | 'array' | 'object'
      description: string
      required?: boolean
    }
  >
  required?: string[]
}

export interface SkillContext {
  sessionId: string
}

// Risk level drives the confirmation policy:
//  - read:   side-effect-free, runs without asking
//  - write:  mutates user data/config, asks before running
//  - system: system-level / dangerous (shell, hosts file), asks before running
export type SkillRisk = 'read' | 'write' | 'system'

// Where a skill came from. 'builtin' is the default assumed by the registry
// when omitted; MCP bridge marks tools 'mcp'; markdown skills from
// ~/.jedi/skills/ are 'custom'.
export type SkillSource = 'builtin' | 'mcp' | 'custom'

// Wire format of a user-defined markdown skill (mirrors the Rust
// CustomSkillDef in src-tauri/src/api/skills.rs). risk stays a plain string
// at the boundary; createCustomSkill narrows it with a fallback.
export interface CustomSkillDef {
  id: string
  name: string
  description: string
  icon: string
  risk: string
  prompt: string
}

export interface Skill {
  id: string
  name: string
  description: string
  icon: string
  enabled: boolean
  autoCallable: boolean
  // Defaults to 'read' when omitted.
  risk?: SkillRisk
  // Defaults to 'builtin' when omitted.
  source?: SkillSource
  execute: (args: any, context: SkillContext) => Promise<any>
  parameters: ParameterSchema
}
