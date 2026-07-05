// src/stores/mcpClient.ts
//
// Third-party MCP client store. Manages user-configured external MCP servers
// (stdio), connects to them through the Rust `mcp_*` commands, and bridges each
// discovered remote tool into the agent's skillRegistry so runAgent exposes it
// to the model. Remote tools are marked write-risk so they go through the
// confirmation gate before running.

import { invoke } from '@tauri-apps/api/core'
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { skillRegistry } from '@/skills/registry'
import type { ParameterSchema, Skill } from '@/skills/types'

export interface McpServerConfig {
  id: string
  name: string
  /** Local stdio command; may be empty for a remote (url) server. */
  command: string
  args: string[]
  /** Extra env vars as [name, value] pairs. */
  env: [string, string][]
  /** Remote MCP server URL (HTTP+SSE). When set, stdio fields are ignored. */
  url?: string
}

export interface McpToolInfo {
  name: string
  description: string
  input_schema: unknown
}

export interface McpConnectedServer {
  id: string
  name: string
  tools: McpToolInfo[]
}

const STORAGE_KEY = 'mcp-servers'

// AI-SDK tool names (and our skill ids) must match ^[a-zA-Z0-9_-]{1,64}$.
function skillIdFor(serverId: string, toolName: string): string {
  const raw = `mcp__${serverId}__${toolName}`
  return raw.replace(/[^a-zA-Z0-9_-]/g, '_').slice(0, 64)
}

export const useMcpClientStore = defineStore('mcpClient', () => {
  const servers = ref<McpServerConfig[]>([])
  const connectedIds = ref<string[]>([])
  const connectingIds = ref<string[]>([])
  const error = ref<string | null>(null)
  // serverId -> skill ids registered for it, so we can unregister on disconnect.
  const registeredSkills = new Map<string, string[]>()

  function loadFromStorage(): void {
    try {
      const saved = localStorage.getItem(STORAGE_KEY)
      if (saved) servers.value = JSON.parse(saved) as McpServerConfig[]
    } catch (e) {
      console.error('Failed to load MCP servers:', e)
    }
  }

  function saveToStorage(): void {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(servers.value))
    } catch (e) {
      console.error('Failed to save MCP servers:', e)
    }
  }

  function addServer(config: McpServerConfig): void {
    const idx = servers.value.findIndex(s => s.id === config.id)
    if (idx >= 0) servers.value[idx] = config
    else servers.value.push(config)
    saveToStorage()
  }

  async function removeServer(id: string): Promise<void> {
    if (connectedIds.value.includes(id)) await disconnect(id)
    servers.value = servers.value.filter(s => s.id !== id)
    saveToStorage()
  }

  function bridgeTools(server: McpConnectedServer): void {
    const ids: string[] = []
    for (const tool of server.tools) {
      const id = skillIdFor(server.id, tool.name)
      const parameters = (tool.input_schema ?? {
        type: 'object',
        properties: {},
      }) as ParameterSchema
      const skill: Skill = {
        id,
        name: tool.name,
        description: `[MCP:${server.name}] ${tool.description || tool.name}`,
        icon: '🔌',
        enabled: true,
        autoCallable: true,
        // Third-party tools are untrusted → confirm before running.
        risk: 'write',
        parameters,
        execute: (args: unknown) =>
          invoke<string>('mcp_call_tool', { id: server.id, tool: tool.name, args }),
      }
      skillRegistry.register(skill)
      ids.push(id)
    }
    registeredSkills.set(server.id, ids)
  }

  function unbridgeTools(serverId: string): void {
    const ids = registeredSkills.get(serverId) ?? []
    ids.forEach(id => skillRegistry.unregister(id))
    registeredSkills.delete(serverId)
  }

  async function connect(id: string): Promise<void> {
    const config = servers.value.find(s => s.id === id)
    if (!config) throw new Error(`未找到 MCP server 配置: ${id}`)
    if (connectingIds.value.includes(id)) return
    error.value = null
    connectingIds.value.push(id)
    try {
      const server = await invoke<McpConnectedServer>('mcp_connect', { config })
      bridgeTools(server)
      if (!connectedIds.value.includes(id)) connectedIds.value.push(id)
    } catch (e: any) {
      error.value = e?.message ?? String(e)
      throw e
    } finally {
      connectingIds.value = connectingIds.value.filter(x => x !== id)
    }
  }

  async function disconnect(id: string): Promise<void> {
    try {
      await invoke('mcp_disconnect', { id })
    } finally {
      unbridgeTools(id)
      connectedIds.value = connectedIds.value.filter(x => x !== id)
    }
  }

  function isConnected(id: string): boolean {
    return connectedIds.value.includes(id)
  }

  function isConnecting(id: string): boolean {
    return connectingIds.value.includes(id)
  }

  return {
    servers,
    connectedIds,
    connectingIds,
    error,
    loadFromStorage,
    saveToStorage,
    addServer,
    removeServer,
    connect,
    disconnect,
    isConnected,
    isConnecting,
  }
})
