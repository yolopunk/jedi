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

export interface Skill {
  id: string
  name: string
  description: string
  icon: string
  enabled: boolean
  autoCallable: boolean
  execute: (args: any, context: SkillContext) => Promise<any>
  parameters: ParameterSchema
}
