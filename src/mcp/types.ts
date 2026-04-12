// src/mcp/types.ts

export interface McpTool {
  name: string
  description: string
  inputSchema: Record<string, any>
}

export interface McpServer {
  id: string
  name: string
  description: string
  tools: McpTool[]

  callTool(toolName: string, args: any): Promise<any>
}
