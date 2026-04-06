// src/skills/podcast.ts

import type { Skill, ParameterSchema } from './types'

const parameters: ParameterSchema = {
  type: 'object',
  properties: {
    operation: { type: 'string', description: 'list, play, search', required: true }
  },
  required: ['operation']
}

async function executePodcast(args: any): Promise<any> {
  return { message: 'Podcast skill coming soon', args }
}

export const podcastSkill: Skill = {
  id: 'podcast',
  name: 'PODCAST',
  description: 'Manage and play podcasts',
  icon: '🎙',
  enabled: false,
  autoCallable: false,
  parameters,
  execute: executePodcast
}
