// src/agent/tools/terminal.ts

import { invoke } from '@tauri-apps/api/core'
import type { ParameterSchema, Tool } from './types'

const parameters: ParameterSchema = {
  type: 'object',
  properties: {
    command: { type: 'string', description: 'Shell command to execute', required: true },
    cwd: { type: 'string', description: 'Working directory', required: false },
  },
  required: ['command'],
}

async function executeTerminal(args: { command: string; cwd?: string }): Promise<string> {
  try {
    return await invoke<string>('execute_command', {
      command: args.command,
      cwd: args.cwd || null,
    })
  } catch (e) {
    throw new Error(`Terminal command failed: ${e}`)
  }
}

export const terminalTool: Tool = {
  id: 'terminal',
  name: 'TERMINAL',
  description: 'Execute system commands in the shell',
  icon: '⌘',
  enabled: true,
  risk: 'system',
  parameters,
  execute: executeTerminal,
}
