// src/mcp/registry.ts

import { hostsMcpServer } from './hosts'
import type { McpServer, McpTool } from './types'

export class McpRegistry {
  private servers: Map<string, McpServer> = new Map()

  register(server: McpServer): void {
    this.servers.set(server.id, server)
  }

  get(id: string): McpServer | undefined {
    return this.servers.get(id)
  }

  list(): McpServer[] {
    return Array.from(this.servers.values())
  }

  getAllTools(): McpTool[] {
    const tools: McpTool[] = []
    for (const server of this.list()) {
      tools.push(...server.tools)
    }
    return tools
  }

  async callTool(serverId: string, toolName: string, args: any): Promise<any> {
    const server = this.get(serverId)
    if (!server) throw new Error(`MCP server not found: ${serverId}`)
    return await server.callTool(toolName, args)
  }
}

export const mcpRegistry = new McpRegistry()

// Register built-in servers
mcpRegistry.register(hostsMcpServer)
