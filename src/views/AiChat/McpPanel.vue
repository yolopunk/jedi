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

function _toggleServer(id: string): void {
  const enabled = mcpStore.isServerEnabled(id)
  mcpStore.toggleServer(id, !enabled)
}

function _selectTool(serverId: string, tool: McpTool): void {
  console.log('Selected tool:', serverId, tool.name)
}
</script>

<style scoped>
.mcp-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: rgba(0, 0, 0, 0.3);
  border-right: 1px solid rgba(0, 255, 255, 0.2);
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid rgba(0, 255, 255, 0.2);
  background: rgba(0, 255, 255, 0.05);
}

.panel-title {
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  font-weight: 700;
  color: #00ffff;
  letter-spacing: 1px;
}

.panel-status {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  color: #00ff88;
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
  background: rgba(0, 255, 255, 0.05);
}

.toggle-indicator {
  width: 12px;
  height: 12px;
  border-radius: 3px;
  border: 2px solid #52525b;
  background: transparent;
  transition: all 0.2s;
}

.server-toggle.enabled .toggle-indicator {
  border-color: #00ff88;
  background: #00ff88;
}

.server-name {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 600;
  color: #a1a1aa;
}

.server-toggle.enabled .server-name {
  color: #e4e4e7;
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
  background: rgba(0, 255, 255, 0.05);
  border-left-color: #00ffff;
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
  color: #a1a1aa;
}

.tool-desc {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  color: #52525b;
  line-height: 1.3;
}

.panel-footer {
  padding: 12px 16px;
  border-top: 1px solid rgba(0, 255, 255, 0.1);
  background: rgba(0, 0, 0, 0.2);
}

.footer-text {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  color: #52525b;
  letter-spacing: 1px;
}

.mcp-servers::-webkit-scrollbar { width: 6px; }
.mcp-servers::-webkit-scrollbar-track { background: transparent; }
.mcp-servers::-webkit-scrollbar-thumb { background: rgba(0, 255, 255, 0.2); border-radius: 3px; }
</style>
