// Tool = an executable capability the model calls directly (function calling).
// This is deliberately separate from "skills": a skill is a SKILL.md
// instruction package with no executable body (see src/skills/types.ts).
// Risk drives the confirmation policy:
//  - read:   side-effect-free, runs without asking
//  - write:  mutates user data/config, asks before running
//  - system: system-level / dangerous (shell, hosts file), asks before running

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

export interface ToolContext {
  sessionId: string
}

export type ToolRisk = 'read' | 'write' | 'system'

// Where a tool came from. 'builtin' is the default assumed by the registry
// when omitted; the MCP bridge marks remote tools 'mcp'.
export type ToolSource = 'builtin' | 'mcp'

export interface Tool {
  id: string
  name: string
  description: string
  icon: string
  enabled: boolean
  // Defaults to 'read' when omitted.
  risk?: ToolRisk
  // Defaults to 'builtin' when omitted.
  source?: ToolSource
  execute: (args: any, context: ToolContext) => Promise<any>
  parameters: ParameterSchema
}
