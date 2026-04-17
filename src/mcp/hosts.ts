// src/mcp/hosts.ts

import { invoke } from '@tauri-apps/api/core'
import type { McpServer, McpTool } from './types'

const tools: McpTool[] = [
  {
    name: 'read_hosts',
    description: 'Read the current hosts file content',
    inputSchema: { type: 'object', properties: {}, required: [] },
  },
  {
    name: 'add_entry',
    description: 'Add a new entry to the hosts file',
    inputSchema: {
      type: 'object',
      properties: {
        ip: { type: 'string', description: 'IP address' },
        hostname: { type: 'string', description: 'Hostname' },
        group: { type: 'string', description: 'Group name (optional)' },
      },
      required: ['ip', 'hostname'],
    },
  },
  {
    name: 'remove_entry',
    description: 'Remove an entry from the hosts file',
    inputSchema: {
      type: 'object',
      properties: {
        hostname: { type: 'string', description: 'Hostname to remove' },
      },
      required: ['hostname'],
    },
  },
  {
    name: 'list_groups',
    description: 'List all hosts groups',
    inputSchema: { type: 'object', properties: {}, required: [] },
  },
]

export const hostsMcpServer: McpServer = {
  id: 'hosts',
  name: 'Hosts Manager',
  description: 'Manage system hosts file entries and groups',
  tools,

  async callTool(toolName: string, args: any): Promise<any> {
    switch (toolName) {
      case 'read_hosts':
        return await invoke('read_hosts')
      case 'add_entry':
        return await invoke('add_hosts_entry', {
          ip: args.ip,
          hostname: args.hostname,
          group: args.group || null,
        })
      case 'remove_entry':
        return await invoke('remove_hosts_entry', { hostname: args.hostname })
      case 'list_groups':
        return await invoke('list_hosts_groups')
      default:
        throw new Error(`Unknown tool: ${toolName}`)
    }
  },
}
