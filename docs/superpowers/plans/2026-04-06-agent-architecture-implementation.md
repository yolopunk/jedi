# Agent Architecture Implementation Plan

> **Status:** ✅ COMPLETED (2026-04-06)

**Goal:** Implement an AI Agent-centric architecture for Jedi app with Skill system, Agent Loop, Provider system, and MCP services.

---

## ✅ Phase 1: Skill System

- [x] `src/skills/types.ts` — Skill types
- [x] `src/skills/registry.ts` — SkillRegistry
- [x] `src/skills/index.ts` — barrel export
- [x] `src/stores/skills.ts` — useSkillsStore
- [x] Built-in skills: terminal, filesystem, hosts, browser, podcast, wallpaper
- [x] `src/views/AiChat/SkillPanel.vue` — UI component
- [x] Integrated into chat page
- [x] Commit: `477c51c`

## ✅ Phase 2: Agent Loop

- [x] `src/agent/types.ts` — Agent types
- [x] `src/agent/loop.ts` — AgentLoop class
- [x] `src/agent/index.ts` — barrel export
- [x] `src/stores/agent.ts` — useAgentStore
- [x] `src/views/AiChat/AgentTrace.vue` — trace panel
- [x] Commit: `6d59bdd`

## ✅ Phase 3: Provider System

- [x] `src/providers/types.ts` — ProviderAdapter
- [x] `src/providers/openai.ts` — OpenAI adapter
- [x] `src/providers/anthropic.ts` — Anthropic adapter
- [x] `src/providers/registry.ts` — ProviderRegistry
- [x] `src/providers/index.ts` — barrel export
- [x] `src/stores/providers.ts` — useProvidersStore
- [x] Commit: `cfd1a46`

## ✅ Phase 4: MCP Service Layer

- [x] `src/mcp/types.ts` — McpServer, McpTool
- [x] `src/mcp/hosts.ts` — Hosts MCP server
- [x] `src/mcp/registry.ts` — McpRegistry
- [x] `src/mcp/index.ts` — barrel export
- [x] `src/stores/mcp.ts` — useMcpStore
- [x] `src/views/AiChat/McpPanel.vue` — tools panel
- [x] Commit: `ca2fb17`

---

## Implementation Summary

**Total commits:** 4 (cfd1a46, 6d59bdd, 477c51c, ca2fb17)
**Files created:** 23 across agent/, skills/, providers/, mcp/, stores/, views/AiChat/

## Next Steps

- Connect Agent Loop to chat messages for autonomous task execution
- Add streaming support for Agent Loop
- Implement confirmation mode UI
- Add more MCP servers (wallpaper, podcast)
- Full LLM integration with Provider system
