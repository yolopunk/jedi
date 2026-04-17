// src/skills/browser.ts

import type { ParameterSchema, Skill } from './types'

const parameters: ParameterSchema = {
  type: 'object',
  properties: {
    operation: { type: 'string', description: 'search, navigate', required: true },
    query: { type: 'string', description: 'Search query or URL', required: true },
  },
  required: ['operation', 'query'],
}

async function executeBrowser(args: any): Promise<any> {
  return { message: 'Browser skill coming soon', args }
}

export const browserSkill: Skill = {
  id: 'browser',
  name: 'BROWSER',
  description: 'Web browsing and search',
  icon: '🌍',
  enabled: false,
  autoCallable: true,
  parameters,
  execute: executeBrowser,
}
