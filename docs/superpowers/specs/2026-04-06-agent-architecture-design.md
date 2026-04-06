---
name: Agent Architecture Design
description: Jedi 应用架构重塑设计 - AI Agent 作为一等公民
type: spec
---

# Agent Architecture Design - Jedi 应用架构重塑

**日期**: 2026-04-06
**状态**: 待实现
**版本**: 1.0

## 概述

本设计文档描述了 Jedi 应用的架构重塑，将 AI Agent 作为一等公民，其他功能模块（Hosts、壁纸、播客）通过 MCP (Model Context Protocol) 为聊天服务。

## 设计目标

1. **Agent 为中心**: AI Agent Loop 是核心功能
2. **即插即用 Provider**: 参考 OpenCode 实现灵活的 LLM Provider 配置系统
3. **Skill 系统**: 内置技能系统，当前阶段为重点
4. **MCP 服务**: 现有功能模块通过 MCP 暴露给 Agent（仅 Hosts）
5. **保持现有体验**: 保留多页面架构，现有页面功能完整

## 架构总览

```
┌─────────────────────────────────────────────────────────────┐
│                     UI Layer (Vue)                          │
│  ┌─────────┐  ┌───────┐  ┌──────────┐  ┌─────────┐     │
│  │  Chat   │  │ Hosts │  │Wallpapers│  │ Podcast │     │
│  └─────────┘  └───────┘  └──────────┘  └─────────┘     │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                Agent Orchestrator Layer                      │
│  ┌──────────┐  ┌─────────┐  ┌──────────┐  ┌─────────┐   │
│  │ Planning │→ │Executor │→ │Reflection│→ │  Tools  │   │
│  └──────────┘  └─────────┘  └──────────┘  └─────────┘   │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│               Provider Abstraction Layer                     │
│  ┌──────────────────────────────────────────────────────┐  │
│  │           Provider Registry                            │  │
│  │  ┌────────┐ ┌─────────┐ ┌────────┐ ┌──────────┐  │  │
│  │  │ OpenAI │ │Anthropic│ │ Google │ │  Custom  │  │  │
│  │  └────────┘ └─────────┘ └────────┘ └──────────┘  │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                    Skills & MCP Layer                        │
│  ┌──────────────────┐      ┌──────────────────────────┐    │
│  │  Skill Registry  │      │    MCP Service Layer     │    │
│  │  • Terminal      │      │  ┌───────────────────┐  │    │
│  │  • Filesystem    │      │  │  Hosts MCP        │  │    │
│  │  • Hosts         │      │  └───────────────────┘  │    │
│  │  • Browser       │      │  (More later)          │    │
│  │  • Podcast       │      └──────────────────────────┘    │
│  │  • Wallpaper     │                                        │
│  └──────────────────┘                                        │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                   Tauri Backend (Rust)                      │
└─────────────────────────────────────────────────────────────┘
```

## 目录结构变更

```
jedi/
├── src/
│   ├── agent/                    # NEW: Agent 核心层
│   │   ├── loop.ts              # Agent Loop 实现
│   │   ├── planner.ts           # 任务规划器
│   │   ├── executor.ts          # 步骤执行器
│   │   ├── reflector.ts         # 反思模块
│   │   └── types.ts             # Agent 类型定义
│   ├── providers/                # NEW: Provider 抽象层
│   │   ├── registry.ts          # Provider 注册中心
│   │   ├── adapter.ts           # Provider 适配器接口
│   │   ├── openai.ts            # OpenAI 兼容适配器
│   │   ├── anthropic.ts         # Anthropic 适配器
│   │   └── config.ts            # Provider 配置 UI
│   ├── mcp/                      # NEW: MCP 服务层
│   │   ├── server.ts            # MCP 服务器接口
│   │   ├── hosts.ts             # Hosts MCP 服务
│   │   └── registry.ts          # MCP 服务注册
│   ├── skills/                   # NEW: Skills 系统（重点！）
│   │   ├── registry.ts          # Skill 注册中心
│   │   ├── executor.ts          # Skill 执行器
│   │   ├── types.ts             # Skill 类型定义
│   │   ├── terminal.ts          # Terminal Skill
│   │   ├── filesystem.ts        # Filesystem Skill
│   │   ├── hosts.ts             # Hosts Skill
│   │   └── browser.ts           # Browser Skill
│   ├── stores/
│   │   ├── agent.ts             # NEW: Agent Store
│   │   ├── providers.ts         # NEW: Providers Store
│   │   ├── mcp.ts               # NEW: MCP Store
│   │   ├── skills.ts            # NEW: Skills Store
│   │   └── aiChat.ts            # 重构：简化为会话管理
│   └── views/AiChat/
│       ├── AgentTrace.vue        # NEW: Agent 执行追踪面板
│       ├── SkillPanel.vue        # NEW: Skill 选择面板
│       ├── McpPanel.vue          # NEW: MCP 工具面板
│       ├── ProviderConfig.vue    # 重构：Provider 配置 UI
│       └── index.vue             # 重构：整合 Agent Loop
└── src-tauri/src/
    ├── agent/                    # NEW: Rust Agent 支持
    │   └── mod.rs
    ├── providers/                # NEW: Rust Provider 层
    │   └── mod.rs
    └── mcp/                      # NEW: Rust MCP 服务
        ├── mod.rs
        └── hosts.rs
```

## 1. Agent Loop 设计

### 状态机

```
INIT → PLANNING → EXECUTING → REFLECTING → (DONE | NEEDS_CLARIFICATION | ERROR)
                    ↑                ↓
                    └────────────────┘
```

### 核心类型定义

```typescript
// src/agent/types.ts

export interface AgentStep {
  id: string
  type: 'think' | 'tool' | 'skill' | 'finish'
  status: 'pending' | 'running' | 'done' | 'error'
  content: string
  result?: any
  error?: string
  timestamp: number
}

export interface AgentState {
  status: 'idle' | 'planning' | 'executing' | 'paused' | 'done' | 'error'
  currentStep: AgentStep | null
  history: AgentStep[]
  confirmationRequired: boolean
  requiresConfirmation: (step: AgentStep) => boolean
}

export interface AgentConfig {
  model: string
  provider: string
  confirmationMode: 'auto' | 'always' | 'dangerous' // 自动 / 总是确认 / 危险操作确认
  maxIterations: number
  temperature: number
}
```

### Agent Loop 流程

1. **Planning 阶段**: LLM 分析用户请求，生成执行计划
2. **Executing 阶段**: 按计划执行步骤，调用 Tools/Skills
3. **Reflecting 阶段**: 评估执行结果，决定下一步
4. **确认模式**: 可配置是否需要用户确认

## 2. Skill 系统设计（当前阶段重点）

### Skill 类型定义

```typescript
// src/skills/types.ts

export interface ParameterSchema {
  type: 'object'
  properties: Record<string, {
    type: 'string' | 'number' | 'boolean' | 'array' | 'object'
    description: string
    required?: boolean
  }>
  required?: string[]
}

export interface SkillContext {
  sessionId: string
  agentState: AgentState
  // 可以访问其他 services
}

export interface Skill {
  id: string
  name: string
  description: string
  icon: string
  enabled: boolean
  // Skill 可以被 Agent 自动调用，也可以被用户手动激活
  autoCallable: boolean
  // 执行函数
  execute: (args: any, context: SkillContext) => Promise<any>
  // 参数 schema（用于 UI 和 Agent 理解）
  parameters: ParameterSchema
}
```

### 内置 Skills

| Skill ID | 名称 | 描述 | 自动调用 |
|----------|------|------|----------|
| terminal | Terminal | 执行系统命令 | true（需确认）|
| filesystem | Filesystem | 读写文件 | true（需确认）|
| hosts | Hosts Manager | 管理 hosts 文件 | true |
| browser | Browser | 网页浏览 | true |
| podcast | Podcast | 播客管理 | false |
| wallpaper | Wallpaper | 壁纸管理 | false |

### Skill Registry

```typescript
// src/skills/registry.ts

export class SkillRegistry {
  private skills: Map<string, Skill> = new Map()

  register(skill: Skill): void {
    this.skills.set(skill.id, skill)
  }

  get(id: string): Skill | undefined {
    return this.skills.get(id)
  }

  list(): Skill[] {
    return Array.from(this.skills.values())
  }

  listEnabled(): Skill[] {
    return this.list().filter(s => s.enabled)
  }

  listAutoCallable(): Skill[] {
    return this.listEnabled().filter(s => s.autoCallable)
  }

  setEnabled(id: string, enabled: boolean): void {
    const skill = this.skills.get(id)
    if (skill) {
      skill.enabled = enabled
    }
  }
}
```

## 3. Provider 系统设计（参考 OpenCode）

### Provider 适配器接口

```typescript
// src/providers/adapter.ts

export interface ProviderAdapter {
  id: string
  name: string
  configSchema: ConfigSchema

  validateConfig(config: ProviderConfig): Promise<boolean>
  listModels(config: ProviderConfig): Promise<Model[]>
  createChatCompletion(
    config: ProviderConfig,
    request: ChatCompletionRequest
  ): Promise<ChatCompletionResponse>
  createChatCompletionStream(
    config: ProviderConfig,
    request: ChatCompletionRequest
  ): AsyncIterable<ChatCompletionChunk>
}
```

### Provider 配置 UI

```
┌─────────────────────────────────────────┐
│  [ Provider Configuration ]              │
├─────────────────────────────────────────┤
│  ┌─────────────────────────────────┐   │
│  │ + Add Provider                   │   │
│  └─────────────────────────────────┘   │
│                                         │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐  │
│  │ OpenAI  │ │Anthropic│ │ Google  │  │
│  │ [v]     │ │  [v]    │ │  [ ]    │  │
│  └─────────┘ └─────────┘ └─────────┘  │
│                                         │
│  Provider: OpenAI                       │
│  ┌─────────────────────────────────┐   │
│  │ Name: My OpenAI                 │   │
│  │ API Key: [••••••••••••] [show] │   │
│  │ Base URL: [https://api.openai…]│   │
│  │ [Test Connection] [Save]        │   │
│  └─────────────────────────────────┘   │
│                                         │
│  Models:                                │
│  ┌─────────────────────────────────┐   │
│  │ [x] gpt-4o                      │   │
│  │ [x] gpt-4-turbo                 │   │
│  │ [ ] gpt-3.5-turbo               │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

## 4. 聊天页面 UI 重新设计

### 三栏布局

```
┌─────────────────────────────────────────────────────────────┐
│  [ H O L O C R O N ]  [Jedi@holocron:~/chat]  [CONNECTED] │
├──────────────┬───────────────────────────┬──────────────────┤
│  SKILLS      │                           │  AGENT TRACE     │
│              │                           │                  │
│  [x] TERMINAL│  Chat Messages Area      │  [12:34:01]      │
│  [x] FILESYS │                           │  » PLANNING       │
│  [x] HOSTS   │  ┌───────────────────┐  │  Analyzing task… │
│  [ ] BROWSER │  │ User message      │  │                  │
│  [ ] PODCAST │  └───────────────────┘  │  [12:34:02]      │
│  [ ] WALLPAP │  ┌───────────────────┐  │  » TOOL: HOSTS   │
│              │  │ AI response       │  │  Reading hosts…   │
│              │  └───────────────────┘  │  ✓ Done           │
│              │                           │                  │
│  MCP TOOLS   │                           │  [12:34:03]      │
│              │  Input Area:             │  » THINK          │
│  [x] Hosts   │  >> [type here...]      │  Okay, I've…      │
│  [ ] Others  │  [Skills] [MCP] [Send]  │                  │
│              │                           │                  │
└──────────────┴───────────────────────────┴──────────────────┘
```

### 输入框快捷按钮

- `[Skills]` - 弹出 Skill 面板，快速启用/禁用技能
- `[MCP]` - 弹出 MCP 工具面板，选择可用的 MCP 工具
- `[Send]` - 发送消息（或者按 Enter）

## 5. MCP 服务设计

### MCP 服务接口

```typescript
// src/mcp/server.ts

export interface McpServer {
  id: string
  name: string
  description: string
  tools: McpTool[]

  callTool(toolName: string, args: any): Promise<any>
}

export interface McpTool {
  name: string
  description: string
  inputSchema: any
}
```

### Hosts MCP 服务

```typescript
// src/mcp/hosts.ts

// Tools:
// - read_hosts - 读取 hosts 文件
// - write_hosts - 写入 hosts 文件
// - add_entry - 添加 hosts 条目
// - remove_entry - 删除 hosts 条目
// - list_groups - 列出分组
```

## 6. 实现优先级

### Phase 1: Skills 系统（最重要！）

1. Skill 注册中心和基础类型
2. 内置 Skills（Terminal、Filesystem、Hosts）
3. Skill 面板 UI
4. 输入框快捷按钮

### Phase 2: Agent Loop

1. Agent Loop 核心逻辑
2. Agent Trace 面板（终端风格日志）
3. 确认模式实现

### Phase 3: Provider 系统

1. Provider Registry
2. Provider 配置 UI（参考 opencode）
3. 模型发现和管理

### Phase 4: MCP 服务（仅 Hosts）

1. MCP 服务接口
2. Hosts MCP 服务实现
3. MCP 工具面板

## 7. 路由保持不变

保持现有多页面架构：

- `/` → 重定向到 `/chat`
- `/chat` - AI Chat（Agent）
- `/hosts` - Hosts Manager
- `/wallpapers` - Wallpaper Manager
- `/podcast` - Podcast Manager

## 8. 技术决策

### 状态管理

- 使用 Pinia stores（现有模式）
- 新增 stores: `agent`, `providers`, `mcp`, `skills`
- 重构 `aiChat` store 专注于会话管理

### 样式风格

- 保持现有科幻/终端风格
- 保持 CRT 扫描线效果
- 保持 R2-D2/BB-8 动画
- 新增面板沿用现有视觉语言

### 后端集成

- Tauri commands 保持现有模式
- 新增 commands 支持 Agent、Skills、MCP
- 复用现有的安全审计和 API Key 管理

## 附录

### 参考项目

- Claude Code - Agent Loop 实现参考
- OpenCode - Provider 配置 UI 参考
- Nanobot - Skill 系统参考
- MCP Specification - Model Context Protocol

### 相关文档

- 现有 AI Chat 设计: `docs/ai-chat/`
- 安全设计: `docs/ai-chat/03-security-review.md`
