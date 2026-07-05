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

export interface Skill {
  id: string
  name: string
  description: string
  icon: string
  enabled: boolean
  autoCallable: boolean
  // Defaults to 'read' when omitted.
  risk?: SkillRisk
  execute: (args: any, context: SkillContext) => Promise<any>
  parameters: ParameterSchema
}
