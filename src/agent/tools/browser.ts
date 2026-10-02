// src/agent/tools/browser.ts

import type { ParameterSchema, Tool } from './types'

const parameters: ParameterSchema = {
  type: 'object',
  properties: {
    operation: { type: 'string', description: 'search, navigate', required: true },
    query: { type: 'string', description: 'Search query or URL', required: true },
  },
  required: ['operation', 'query'],
}

async function executeBrowser(args: any): Promise<any> {
  return { message: 'Browser tool coming soon', args }
}

export const browserTool: Tool = {
  id: 'browser',
  name: 'BROWSER',
  description: 'Web browsing and search',
  icon: '🌍',
  enabled: false,
  parameters,
  execute: executeBrowser,
}
