// src/stores/mcp.ts

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { mcpRegistry } from '@/mcp/registry'

export const useMcpStore = defineStore('mcp', () => {
  const enabledServers = ref<string[]>(['hosts'])
  const mcpPanelOpen = ref(false)
  const callingTool = ref<string | null>(null)

  const servers = computed(() => mcpRegistry.list())
  const enabledServersList = computed(() =>
    servers.value.filter(s => enabledServers.value.includes(s.id))
  )
  const allTools = computed(() => mcpRegistry.getAllTools())

  function toggleServer(id: string, enabled: boolean): void {
    if (enabled) {
      if (!enabledServers.value.includes(id)) enabledServers.value.push(id)
    } else {
      enabledServers.value = enabledServers.value.filter(s => s !== id)
    }
  }

  function isServerEnabled(id: string): boolean {
    return enabledServers.value.includes(id)
  }

  function toggleMcpPanel(): void {
    mcpPanelOpen.value = !mcpPanelOpen.value
  }

  async function callTool(serverId: string, toolName: string, args: any): Promise<any> {
    callingTool.value = `${serverId}.${toolName}`
    try {
      const result = await mcpRegistry.callTool(serverId, toolName, args)
      return result
    } finally {
      callingTool.value = null
    }
  }

  return {
    enabledServers, mcpPanelOpen, callingTool,
    servers, enabledServersList, allTools,
    toggleServer, isServerEnabled, toggleMcpPanel, callTool
  }
})
