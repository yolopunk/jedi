// src/skills/filesystem.ts

import { invoke } from '@tauri-apps/api/core'
import type { ParameterSchema, Skill } from './types'

const parameters: ParameterSchema = {
  type: 'object',
  properties: {
    operation: { type: 'string', description: 'read, write, exists', required: true },
    path: { type: 'string', description: 'File path', required: true },
    content: { type: 'string', description: 'Content for write', required: false },
  },
  required: ['operation', 'path'],
}

async function executeFilesystem(args: any): Promise<any> {
  const { operation, path, content } = args
  switch (operation) {
    case 'read':
      return await invoke('read_file', { path })
    case 'write':
      return await invoke('write_file', { path, content })
    case 'exists':
      return await invoke('file_exists', { path })
    default:
      throw new Error(`Unknown operation: ${operation}`)
  }
}

export const filesystemSkill: Skill = {
  id: 'filesystem',
  name: 'FILESYS',
  description: 'Read and write files',
  icon: '📁',
  enabled: true,
  autoCallable: true,
  parameters,
  execute: executeFilesystem,
}
