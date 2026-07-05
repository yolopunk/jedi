<template>
  <div class="mcp-panel">
    <div class="panel-header">
      <span class="panel-title">// MCP TOOLS</span>
      <span class="panel-status">READY</span>
    </div>
    <div class="mcp-servers">
      <div v-for="server in mcpStore.servers" :key="server.id" class="server-section">
        <div class="server-header">
          <div
            class="server-toggle"
            :class="{ enabled: mcpStore.isServerEnabled(server.id) }"
            @click="toggleServer(server.id)"
          >
            <div class="toggle-indicator"></div>
            <span class="server-name">{{ server.name }}</span>
          </div>
        </div>
        <div v-if="mcpStore.isServerEnabled(server.id)" class="tools-list">
          <div
            v-for="tool in server.tools"
            :key="tool.name"
            class="tool-item"
            @click="selectTool(server.id, tool)"
          >
            <span class="tool-icon">⚡</span>
            <div class="tool-info">
              <span class="tool-name">{{ tool.name }}</span>
              <span class="tool-desc">{{ tool.description }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
    <div class="panel-footer">
      <span class="footer-text">AGENT CAN USE THESE TOOLS</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { McpTool } from '@/mcp/types'
import { useMcpStore } from '@/stores/mcp'

const mcpStore = useMcpStore()

function toggleServer(id: string): void {
  const enabled = mcpStore.isServerEnabled(id)
  mcpStore.toggleServer(id, !enabled)
}

function selectTool(serverId: string, tool: McpTool): void {
  console.log('Selected tool:', serverId, tool.name)
}
</script>

<style scoped>
.mcp-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background:rgb(var(--ink-rgb) / 0.3);
  border-right: 1px solid rgb(var(--accent-rgb) / 0.2);
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid rgb(var(--accent-rgb) / 0.2);
  background: rgb(var(--accent-rgb) / 0.05);
}

.panel-title {
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  font-weight: 700;
  color: var(--accent);
  letter-spacing: 1px;
}

.panel-status {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  color: var(--success);
}

.mcp-servers {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.server-section {
  margin-bottom: 12px;
}

.server-header {
  padding: 8px 4px;
}

.server-toggle {
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  padding: 6px 8px;
  border-radius: 4px;
  transition: background 0.2s;
}

.server-toggle:hover {
  background: rgb(var(--accent-rgb) / 0.05);
}

.toggle-indicator {
  width: 12px;
  height: 12px;
  border-radius: 3px;
  border: 2px solid var(--text-subtle);
  background: transparent;
  transition: all 0.2s;
}

.server-toggle.enabled .toggle-indicator {
  border-color: var(--success);
  background: var(--success);
}

.server-name {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
}

.server-toggle.enabled .server-name {
  color: var(--border);
}

.tools-list {
  padding-left: 22px;
}

.tool-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 8px 10px;
  margin-bottom: 4px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.2s;
  border-left: 2px solid transparent;
}

.tool-item:hover {
  background: rgb(var(--accent-rgb) / 0.05);
  border-left-color: var(--accent);
}

.tool-icon {
  font-size: 12px;
  margin-top: 2px;
}

.tool-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.tool-name {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
}

.tool-desc {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  color: var(--text-subtle);
  line-height: 1.3;
}

.panel-footer {
  padding: 12px 16px;
  border-top: 1px solid rgb(var(--accent-rgb) / 0.1);
  background:rgb(var(--ink-rgb) / 0.2);
}

.footer-text {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  color: var(--text-subtle);
  letter-spacing: 1px;
}

.mcp-servers::-webkit-scrollbar { width: 6px; }
.mcp-servers::-webkit-scrollbar-track { background: transparent; }
.mcp-servers::-webkit-scrollbar-thumb { background: rgb(var(--accent-rgb) / 0.2); border-radius: 3px; }
</style>
