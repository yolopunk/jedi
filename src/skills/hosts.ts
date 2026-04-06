// src/skills/hosts.ts

import type { Skill, ParameterSchema } from './types'
import { invoke } from '@tauri-apps/api/core'

const parameters: ParameterSchema = {
  type: 'object',
  properties: {
    operation: { type: 'string', description: 'read, add_entry, remove_entry, list_groups', required: true },
    ip: { type: 'string', description: 'IP address', required: false },
    hostname: { type: 'string', description: 'Hostname', required: false },
    group: { type: 'string', description: 'Group name', required: false }
  },
  required: ['operation']
}

async function executeHosts(args: any): Promise<any> {
  const { operation, ip, hostname, group } = args
  switch (operation) {
    case 'read': return await invoke('read_hosts')
    case 'add_entry':
      if (!ip || !hostname) throw new Error('ip and hostname required')
      return await invoke('add_hosts_entry', { ip, hostname, group: group || null })
    case 'remove_entry':
      if (!hostname) throw new Error('hostname required')
      return await invoke('remove_hosts_entry', { hostname })
    case 'list_groups': return await invoke('list_hosts_groups')
    default: throw new Error(`Unknown operation: ${operation}`)
  }
}

export const hostsSkill: Skill = {
  id: 'hosts',
  name: 'HOSTS_MGR',
  description: 'Manage system hosts file',
  icon: '🌐',
  enabled: true,
  autoCallable: true,
  parameters,
  execute: executeHosts
}
