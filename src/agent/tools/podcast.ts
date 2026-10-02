// src/agent/tools/podcast.ts

import type { ParameterSchema, Tool } from './types'

const parameters: ParameterSchema = {
  type: 'object',
  properties: {
    operation: { type: 'string', description: 'list, play, search', required: true },
  },
  required: ['operation'],
}

async function executePodcast(args: any): Promise<any> {
  return { message: 'Podcast tool coming soon', args }
}

export const podcastTool: Tool = {
  id: 'podcast',
  name: 'PODCAST',
  description: 'Manage and play podcasts',
  icon: '🎙',
  enabled: false,
  parameters,
  execute: executePodcast,
}
