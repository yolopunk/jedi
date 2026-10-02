// src/agent/tools/registry.ts
//
// In-memory registry of executable agent tools. Builtins register at module
// load; MCP-bridged tools register at connect time (stores/mcpClient.ts).
// The user-facing enabled flags are persisted by stores/tools.ts and applied
// here in both directions via setEnabled.

import { browserTool } from './browser'
import { filesystemTool } from './filesystem'
import { hostsTool } from './hosts'
import { memoryTool } from './memory'
import { podcastTool } from './podcast'
import { terminalTool } from './terminal'
import type { Tool } from './types'
import { wallpaperTool } from './wallpaper'
import { webFetchTool, webSearchTool } from './web'

export class ToolRegistry {
  private tools: Map<string, Tool> = new Map()

  register(tool: Tool): void {
    if (!tool.source) tool.source = 'builtin'
    this.tools.set(tool.id, tool)
  }

  unregister(id: string): void {
    this.tools.delete(id)
  }

  get(id: string): Tool | undefined {
    return this.tools.get(id)
  }

  list(): Tool[] {
    return Array.from(this.tools.values())
  }

  listEnabled(): Tool[] {
    return this.list().filter(t => t.enabled)
  }

  setEnabled(id: string, enabled: boolean): void {
    const tool = this.tools.get(id)
    if (tool) {
      tool.enabled = enabled
    }
  }
}

export const toolRegistry = new ToolRegistry()

// Register built-in tools
toolRegistry.register(terminalTool)
toolRegistry.register(filesystemTool)
toolRegistry.register(hostsTool)
toolRegistry.register(browserTool)
toolRegistry.register(podcastTool)
toolRegistry.register(wallpaperTool)
toolRegistry.register(webSearchTool)
toolRegistry.register(webFetchTool)
toolRegistry.register(memoryTool)

// Set initial enabled states (stubs start disabled)
toolRegistry.setEnabled('browser', false)
toolRegistry.setEnabled('podcast', false)
toolRegistry.setEnabled('wallpaper', false)
