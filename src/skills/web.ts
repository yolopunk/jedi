// src/skills/web.ts
//
// Web access tools for the AI agent, mirroring what Claude Code / Codex expose:
// a search tool (find pages) and a fetch tool (read one page). Both call into the
// Rust backend (reqwest), so they bypass the webview CORS policy and can reach
// live data such as weather, docs, or news.

import { invoke } from '@tauri-apps/api/core'
import type { ParameterSchema, Skill } from './types'

// ==================== web_search ====================

const searchParameters: ParameterSchema = {
  type: 'object',
  properties: {
    query: { type: 'string', description: 'The search query', required: true },
    max_results: {
      type: 'number',
      description: 'Maximum number of results to return (1-10, default 5)',
      required: false,
    },
  },
  required: ['query'],
}

interface SearchHit {
  title: string
  url: string
  snippet: string
}

async function executeWebSearch(args: {
  query: string
  max_results?: number
}): Promise<SearchHit[]> {
  return invoke<SearchHit[]>('web_search', {
    query: args.query,
    maxResults: args.max_results ?? null,
  })
}

export const webSearchSkill: Skill = {
  id: 'web_search',
  name: 'WEB_SEARCH',
  description:
    'Search the web (DuckDuckGo) for up-to-date information. Returns a list of results with title, URL, and snippet. Use this to find pages, then read one with web_fetch.',
  icon: '🔎',
  enabled: true,
  autoCallable: true,
  parameters: searchParameters,
  execute: executeWebSearch,
}

// ==================== web_fetch ====================

const fetchParameters: ParameterSchema = {
  type: 'object',
  properties: {
    url: {
      type: 'string',
      description: 'The absolute URL to fetch (must start with http:// or https://)',
      required: true,
    },
    max_chars: {
      type: 'number',
      description: 'Maximum characters of body text to return (default 8000)',
      required: false,
    },
  },
  required: ['url'],
}

interface WebFetchResult {
  url: string
  status: number
  content_type: string
  title?: string
  text: string
  truncated: boolean
}

async function executeWebFetch(args: { url: string; max_chars?: number }): Promise<WebFetchResult> {
  return invoke<WebFetchResult>('web_fetch', {
    url: args.url,
    maxChars: args.max_chars ?? null,
  })
}

export const webFetchSkill: Skill = {
  id: 'web_fetch',
  name: 'WEB_FETCH',
  description:
    'Fetch a single URL and return its readable text content (HTML is stripped to plain text). Use for reading a specific page, an API/JSON endpoint, or live data like https://wttr.in/Hangzhou for weather.',
  icon: '🌐',
  enabled: true,
  autoCallable: true,
  parameters: fetchParameters,
  execute: executeWebFetch,
}
