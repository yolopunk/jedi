// src/skills/memory.ts
//
// Cross-session memory tool for the AI agent. Lets the model remember long-term
// user preferences and configuration and recall them in later chats. Backed by a
// flat key/value store in ~/.jedi/agent_memory.json via the Rust `memory_*`
// commands. Only touches Jedi's own private storage, so it is friction-free.

import { invoke } from '@tauri-apps/api/core'
import type { ParameterSchema, Skill } from './types'

const memoryParameters: ParameterSchema = {
  type: 'object',
  properties: {
    action: {
      type: 'string',
      description: "Operation to perform: 'save', 'recall', 'list', or 'delete'",
      required: true,
    },
    key: {
      type: 'string',
      description: 'Memory key, e.g. preferred_wallpaper. Required for save / recall / delete.',
      required: false,
    },
    value: {
      type: 'string',
      description: 'Memory content to store. Required for save.',
      required: false,
    },
  },
  required: ['action'],
}

interface MemoryEntry {
  key: string
  value: string
}

async function executeMemory(args: {
  action: string
  key?: string
  value?: string
}): Promise<string> {
  const action = (args.action || '').toLowerCase()
  switch (action) {
    case 'save': {
      if (!args.key || args.value === undefined) return '错误：save 需要 key 和 value'
      return invoke<string>('memory_save', { key: args.key, value: args.value })
    }
    case 'recall': {
      if (!args.key) return '错误：recall 需要 key'
      return invoke<string>('memory_recall', { key: args.key })
    }
    case 'list': {
      const entries = await invoke<MemoryEntry[]>('memory_list')
      if (!entries.length) return '（暂无记忆）'
      return entries.map(e => `${e.key}: ${e.value}`).join('\n')
    }
    case 'delete': {
      if (!args.key) return '错误：delete 需要 key'
      return invoke<string>('memory_delete', { key: args.key })
    }
    default:
      return `未知操作：${args.action}（应为 save / recall / list / delete）`
  }
}

export const memorySkill: Skill = {
  id: 'memory',
  name: 'MEMORY',
  description:
    'Remember long-term user preferences and configuration across chat sessions. ' +
    'action=save stores a key/value pair; action=recall retrieves a value by key; ' +
    'action=list shows everything remembered; action=delete removes a key. ' +
    'Use this whenever the user asks you to remember something for later.',
  icon: '🧠',
  enabled: true,
  autoCallable: true,
  parameters: memoryParameters,
  execute: executeMemory,
}
