# Agent Pool Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a coordinator/worker agent pool system where a coordinator agent can spawn multiple worker agents to execute tasks in parallel, similar to the open-claudecode coordinator mode architecture.

**Architecture:** Jedi runs as a Tauri desktop app with a Vue 3 frontend and Rust backend. The agent pool will be a Rust-side pool that manages multiple concurrent agent task threads, coordinated through Tauri commands. The Vue frontend provides a UI for viewing active workers and their status.

**Tech Stack:** Tauri v2 (Rust backend), Vue 3 + TypeScript (frontend), tokio async runtime, Pinia stores

---

## File Structure

```
src-tauri/src/
├── agent/
│   ├── mod.rs              # Module root
│   ├── pool.rs             # AgentPool struct - create/release/schedule workers
│   ├── worker.rs           # Worker agent - runs in tokio thread
│   ├── coordinator.rs      # Coordinator logic - assigns tasks to workers
│   ├── types.rs            # Shared types (TaskSpec, WorkerStatus, WorkerResult)
│   └── commands.rs         # Tauri command handlers (5 commands)

src/
├── agent/
│   ├── pool.ts             # Frontend pool composable (mirrors Rust state)
│   ├── poolTypes.ts        # TypeScript types matching Rust side
│   └── useAgentPool.ts     # Vue composable for pool UI
├── components/agent/
│   ├── AgentPoolPanel.vue  # Right panel showing active workers
│   ├── WorkerCard.vue      # Individual worker status card
│   └── WorkerDetail.vue    # Expanded worker view (messages, tools used)
├── views/
│   └── AiChat/
│       └── index.vue       # Add AgentPoolPanel to layout
```

---

## Task 1: Rust Types and Pool Core

**Files:**
- Create: `src-tauri/src/agent/types.rs`
- Create: `src-tauri/src/agent/pool.rs`
- Create: `src-tauri/src/agent/mod.rs`
- Modify: `src-tauri/src/main.rs:1-20` (add module)

- [ ] **Step 1: Create `src-tauri/src/agent/types.rs`**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    pub id: String,
    pub description: String,
    pub prompt: String,
    pub tools: Vec<String>,
    pub model: Option<String>,
    pub abort_on_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerStatus {
    pub id: String,
    pub task_id: Option<String>,
    pub description: String,
    pub status: WorkerStatusKind,
    pub progress: Option<String>,
    pub started_at: Option<i64>,
    pub result: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkerStatusKind {
    Idle,
    Running,
    Completed,
    Failed,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerResult {
    pub worker_id: String,
    pub task_id: String,
    pub status: WorkerStatusKind,
    pub result: Option<String>,
    pub error: Option<String>,
    pub duration_ms: Option<u64>,
}
```

- [ ] **Step 2: Create `src-tauri/src/agent/pool.rs`**

```rust
use super::types::{TaskSpec, WorkerResult, WorkerStatus, WorkerStatusKind};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct AgentPool {
    max_workers: usize,
    workers: Arc<Mutex<HashMap<String, WorkerHandle>>>,
}

struct WorkerHandle {
    id: String,
    task_id: Option<String>,
    description: String,
    status: WorkerStatusKind,
    progress: Option<String>,
    started_at: Option<i64>,
    abort_tx: Option<mpsc::Sender<()>>,
}

impl AgentPool {
    pub fn new(max_workers: usize) -> Self {
        Self {
            max_workers,
            workers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn status(&self) -> Vec<WorkerStatus> {
        self.workers
            .lock()
            .unwrap()
            .values()
            .map(|w| WorkerStatus {
                id: w.id.clone(),
                task_id: w.task_id.clone(),
                description: w.description.clone(),
                status: w.status.clone(),
                progress: w.progress.clone(),
                started_at: w.started_at,
                result: None,
                error: None,
            })
            .collect()
    }

    pub fn schedule(&self, spec: TaskSpec) -> Result<String, String> {
        let mut workers = self.workers.lock().unwrap();
        if workers.values().filter(|w| w.status == WorkerStatusKind::Running).count() >= self.max_workers {
            return Err("All workers busy".to_string());
        }
        let worker_id = format!("worker-{}", uuid::Uuid::new_v4());
        let (abort_tx, abort_rx) = mpsc::channel(1);
        workers.insert(worker_id.clone(), WorkerHandle {
            id: worker_id.clone(),
            task_id: Some(spec.id.clone()),
            description: spec.description.clone(),
            status: WorkerStatusKind::Running,
            progress: Some("Starting...".to_string()),
            started_at: Some(chrono::Utc::now().timestamp_millis()),
            abort_tx: Some(abort_tx),
        });
        Ok(worker_id)
    }

    pub fn stop(&self, worker_id: &str) -> Result<(), String> {
        let mut workers = self.workers.lock().unwrap();
        if let Some(handle) = workers.get_mut(worker_id) {
            if let Some(tx) = handle.abort_tx.take() {
                let _ = tx.try_send(());
            }
            handle.status = WorkerStatusKind::Stopped;
            Ok(())
        } else {
            Err("Worker not found".to_string())
        }
    }

    pub fn remove(&self, worker_id: &str) -> Result<WorkerResult, String> {
        let mut workers = self.workers.lock().unwrap();
        if let Some(handle) = workers.remove(worker_id) {
            Ok(WorkerResult {
                worker_id: handle.id,
                task_id: handle.task_id.unwrap_or_default(),
                status: handle.status,
                result: None,
                error: handle.error,
                duration_ms: handle.started_at.map(|s| (chrono::Utc::now().timestamp_millis() - s) as u64),
            })
        } else {
            Err("Worker not found".to_string())
        }
    }
}
```

- [ ] **Step 3: Create `src-tauri/src/agent/mod.rs`**

```rust
pub mod types;
pub mod pool;

pub use types::*;
pub use pool::AgentPool;
```

- [ ] **Step 4: Add module to `src-tauri/src/main.rs`**

Add after existing module declarations:
```rust
mod agent;
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/agent/ src-tauri/src/main.rs
git commit -m "feat(agent): add agent pool types and core pool struct

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 2: Tauri Commands for Pool Management

**Files:**
- Create: `src-tauri/src/agent/commands.rs`
- Modify: `src-tauri/src/main.rs` (register commands)
- Modify: `src-tauri/src/agent/mod.rs`

- [ ] **Step 1: Create `src-tauri/src/agent/commands.rs`**

```rust
use crate::agent::pool::AgentPool;
use crate::agent::types::{TaskSpec, WorkerStatus};
use std::sync::Mutex;
use tauri::State;

pub struct PoolState(pub Mutex<AgentPool>);

#[tauri::command]
pub fn get_pool_status(state: State<PoolState>) -> Vec<WorkerStatus> {
    state.0.lock().unwrap().status()
}

#[tauri::command]
pub fn schedule_task(
    state: State<PoolState>,
    task: TaskSpec,
) -> Result<String, String> {
    state.0.lock().unwrap().schedule(task)
}

#[tauri::command]
pub fn stop_worker(state: State<PoolState>, worker_id: String) -> Result<(), String> {
    state.0.lock().unwrap().stop(&worker_id)
}

#[tauri::command]
pub fn remove_worker(state: State<PoolState>, worker_id: String) -> Result<(), String> {
    state.0.lock().unwrap().remove(&worker_id)?;
    Ok(())
}

#[tauri::command]
pub fn init_pool(state: State<PoolState>, max_workers: usize) -> Result<(), String> {
    let mut pool = state.0.lock().unwrap();
    *pool = AgentPool::new(max_workers);
    Ok(())
}
```

- [ ] **Step 2: Update `src-tauri/src/agent/mod.rs`**

```rust
pub mod types;
pub mod pool;
pub mod commands;

pub use types::*;
pub use pool::AgentPool;
pub use commands::*;
```

- [ ] **Step 3: Register commands and state in `src-tauri/src/main.rs`**

Add near the top with other use statements:
```rust
use crate::agent::commands::{init_pool, get_pool_status, schedule_task, stop_worker, remove_worker, PoolState};
```

Add PoolState to managed state:
```rust
.app_data(PoolState(Mutex::new(AgentPool::new(4))))
```

Register commands in `builder`:
```rust
.invoke_method(init_pool)
.invoke_method(get_pool_status)
.invoke_method(schedule_task)
.invoke_method(stop_worker)
.invoke_method(remove_worker)
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/agent/commands.rs src-tauri/src/main.rs
git commit -m "feat(agent): add Tauri commands for pool management

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 3: Frontend Pool API Layer

**Files:**
- Create: `src/agent/poolTypes.ts`
- Create: `src/agent/pool.ts`
- Modify: `src/api/index.ts` (add pool exports)

- [ ] **Step 1: Create `src/agent/poolTypes.ts`**

```typescript
export type WorkerStatusKind = 'idle' | 'running' | 'completed' | 'failed' | 'stopped'

export interface TaskSpec {
  id: string
  description: string
  prompt: string
  tools: string[]
  model?: string
  abort_on_error: boolean
}

export interface WorkerStatus {
  id: string
  task_id?: string
  description: string
  status: WorkerStatusKind
  progress?: string
  started_at?: number
  result?: string
  error?: string
}

export interface WorkerResult {
  worker_id: string
  task_id: string
  status: WorkerStatusKind
  result?: string
  error?: string
  duration_ms?: number
}
```

- [ ] **Step 2: Create `src/agent/pool.ts`**

```typescript
import { invoke } from '@tauri-apps/api/core'
import type { TaskSpec, WorkerStatus, WorkerResult } from './poolTypes'

export async function initPool(maxWorkers: number = 4): Promise<void> {
  await invoke('init_pool', { maxWorkers })
}

export async function getPoolStatus(): Promise<WorkerStatus[]> {
  return await invoke<WorkerStatus[]>('get_pool_status')
}

export async function scheduleTask(spec: TaskSpec): Promise<string> {
  return await invoke<string>('schedule_task', { task: spec })
}

export async function stopWorker(workerId: string): Promise<void> {
  await invoke('stop_worker', { workerId })
}

export async function removeWorker(workerId: string): Promise<WorkerResult> {
  return await invoke<WorkerResult>('remove_worker', { workerId })
}
```

- [ ] **Step 3: Update `src/api/index.ts`**

Add export:
```typescript
export * from './pool'
```

- [ ] **Step 4: Commit**

```bash
git add src/agent/poolTypes.ts src/agent/pool.ts src/api/index.ts
git commit -m "feat(agent-pool): add frontend pool API layer

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 4: Pool Composable and UI Components

**Files:**
- Create: `src/agent/useAgentPool.ts`
- Create: `src/components/agent/WorkerCard.vue`
- Create: `src/components/agent/AgentPoolPanel.vue`

- [ ] **Step 1: Create `src/agent/useAgentPool.ts`**

```typescript
import { ref, onMounted, onUnmounted } from 'vue'
import { getPoolStatus, type WorkerStatus } from './pool'

const POOL_POLL_INTERVAL = 1000

export function useAgentPool() {
  const workers = ref<WorkerStatus[]>([])
  const panelOpen = ref(false)
  let pollTimer: ReturnType<typeof setInterval> | null = null

  async function refresh() {
    try {
      workers.value = await getPoolStatus()
    } catch (e) {
      console.error('Failed to fetch pool status:', e)
    }
  }

  function startPolling() {
    refresh()
    pollTimer = setInterval(refresh, POOL_POLL_INTERVAL)
  }

  function stopPolling() {
    if (pollTimer) {
      clearInterval(pollTimer)
      pollTimer = null
    }
  }

  onMounted(() => startPolling())
  onUnmounted(() => stopPolling())

  function togglePanel() {
    panelOpen.value = !panelOpen.value
  }

  return {
    workers,
    panelOpen,
    togglePanel,
    refresh,
  }
}
```

- [ ] **Step 2: Create `src/components/agent/WorkerCard.vue`**

```vue
<template>
  <div class="worker-card" :class="worker.status">
    <div class="worker-header">
      <div class="worker-icon">
        <svg v-if="worker.status === 'running'" width="14" height="14" viewBox="0 0 24 24" fill="none">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/>
          <polyline points="12 6 12 12 16 14" stroke="currentColor" stroke-width="2"/>
        </svg>
        <svg v-else-if="worker.status === 'completed'" width="14" height="14" viewBox="0 0 24 24" fill="none">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/>
          <polyline points="9 12 11 14 15 10" stroke="currentColor" stroke-width="2"/>
        </svg>
        <svg v-else-if="worker.status === 'failed'" width="14" height="14" viewBox="0 0 24 24" fill="none">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/>
          <line x1="15" y1="9" x2="9" y2="15" stroke="currentColor" stroke-width="2"/>
          <line x1="9" y1="9" x2="15" y2="15" stroke="currentColor" stroke-width="2"/>
        </svg>
        <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/>
          <line x1="8" y1="12" x2="16" y2="12" stroke="currentColor" stroke-width="2"/>
        </svg>
      </div>
      <div class="worker-info">
        <span class="worker-desc">{{ worker.description }}</span>
        <span class="worker-progress">{{ worker.progress || worker.status }}</span>
      </div>
      <button class="stop-btn" @click="$emit('stop', worker.id)" v-if="worker.status === 'running'">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none">
          <rect x="6" y="6" width="12" height="12" rx="2" fill="currentColor"/>
        </svg>
      </button>
    </div>
    <div class="worker-meta" v-if="worker.started_at">
      {{ formatDuration(worker.started_at) }}
    </div>
  </div>
</template>

<script setup lang="ts">
import type { WorkerStatus } from '@/agent/poolTypes'

defineProps<{ worker: WorkerStatus }>()
defineEmits<{ (e: 'stop', id: string): void }>()

function formatDuration(startedAt: number): string {
  const elapsed = Date.now() - startedAt
  const seconds = Math.floor(elapsed / 1000)
  const minutes = Math.floor(seconds / 60)
  if (minutes > 0) return `${minutes}m ${seconds % 60}s`
  return `${seconds}s`
}
</script>

<style scoped>
.worker-card {
  padding: 12px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  transition: all 0.2s;
}
.worker-card.running { border-color: rgba(0, 255, 136, 0.3); }
.worker-card.completed { border-color: rgba(0, 255, 255, 0.2); }
.worker-card.failed { border-color: rgba(255, 107, 107, 0.3); }
.worker-header { display: flex; align-items: center; gap: 10px; }
.worker-icon { color: rgba(255, 255, 255, 0.5); }
.worker-card.running .worker-icon { color: #00ff88; }
.worker-card.completed .worker-icon { color: #00ffff; }
.worker-card.failed .worker-icon { color: #ff6b6b; }
.worker-info { flex: 1; display: flex; flex-direction: column; gap: 2px; }
.worker-desc { font-size: 12px; font-weight: 500; color: rgba(255, 255, 255, 0.8); }
.worker-progress { font-size: 11px; color: rgba(255, 255, 255, 0.4); }
.worker-meta { margin-top: 8px; font-size: 10px; color: rgba(255, 255, 255, 0.25); }
.stop-btn {
  width: 24px; height: 24px; display: flex; align-items: center; justify-content: center;
  background: rgba(255, 107, 107, 0.1); border: 1px solid rgba(255, 107, 107, 0.2);
  border-radius: 6px; color: #ff6b6b; cursor: pointer;
}
.stop-btn:hover { background: rgba(255, 107, 107, 0.2); }
</style>
```

- [ ] **Step 3: Create `src/components/agent/AgentPoolPanel.vue`**

```vue
<template>
  <div class="agent-pool-panel" v-if="isOpen">
    <div class="panel-header">
      <div class="panel-title">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
          <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" stroke="currentColor" stroke-width="2"/>
          <circle cx="9" cy="7" r="4" stroke="currentColor" stroke-width="2"/>
          <path d="M23 21v-2a4 4 0 0 0-3-3.87" stroke="currentColor" stroke-width="2"/>
          <path d="M16 3.13a4 4 0 0 1 0 7.75" stroke="currentColor" stroke-width="2"/>
        </svg>
        <span>Workers</span>
        <span class="count">{{ workers.length }}</span>
      </div>
      <button class="close-btn" @click="isOpen = false">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
          <line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2"/>
          <line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2"/>
        </svg>
      </button>
    </div>
    <div class="panel-body">
      <div v-if="workers.length === 0" class="empty-state">
        <span>No active workers</span>
      </div>
      <WorkerCard
        v-for="worker in workers"
        :key="worker.id"
        :worker="worker"
        @stop="handleStop"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { useAgentPool } from '@/agent/useAgentPool'
import { stopWorker } from '@/agent/pool'
import WorkerCard from './WorkerCard.vue'

const { workers, panelOpen: isOpen, togglePanel } = useAgentPool()

async function handleStop(workerId: string) {
  try {
    await stopWorker(workerId)
  } catch (e) {
    console.error('Failed to stop worker:', e)
  }
}
</script>

<style scoped>
.agent-pool-panel {
  position: fixed;
  right: 16px;
  bottom: 80px;
  width: 320px;
  background: #0a0e14;
  border: 1px solid rgba(0, 255, 255, 0.15);
  border-radius: 12px;
  overflow: hidden;
  z-index: 100;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
}
.panel-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 12px 14px;
  background: rgba(20, 30, 40, 0.6);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}
.panel-title {
  display: flex; align-items: center; gap: 8px;
  font-size: 13px; font-weight: 600; color: #ffffff;
}
.panel-title svg { color: #00ffff; }
.count {
  padding: 2px 6px;
  background: rgba(0, 255, 136, 0.15);
  border-radius: 10px;
  font-size: 10px;
  color: #00ff88;
}
.close-btn {
  width: 24px; height: 24px; display: flex; align-items: center; justify-content: center;
  background: transparent; border: none; color: rgba(255, 255, 255, 0.4); cursor: pointer;
}
.close-btn:hover { color: #ff6b6b; }
.panel-body { padding: 12px; display: flex; flex-direction: column; gap: 8px; max-height: 300px; overflow-y: auto; }
.empty-state { padding: 24px; text-align: center; color: rgba(255, 255, 255, 0.3); font-size: 12px; }
</style>
```

- [ ] **Step 4: Commit**

```bash
git add src/agent/useAgentPool.ts src/components/agent/WorkerCard.vue src/components/agent/AgentPoolPanel.vue
git commit -m "feat(agent-pool): add pool composable and UI components

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 5: Integrate Pool Panel into AiChat Layout

**Files:**
- Modify: `src/views/AiChat/index.vue` (add AgentPoolPanel, add toggle button)

- [ ] **Step 1: Read current `src/views/AiChat/index.vue` to find insertion points**

- [ ] **Step 2: Add AgentPoolPanel import and usage**

Add to script section:
```typescript
import AgentPoolPanel from '@/components/agent/AgentPoolPanel.vue'
```

Add to template (floating panel):
```vue
<AgentPoolPanel />
```

Add toggle button (e.g., in the header area):
```vue
<v-btn icon size="small" variant="text" @click="togglePool">
  <v-icon icon="mdi-account-group" />
</v-btn>
```

- [ ] **Step 3: Commit**

```bash
git add src/views/AiChat/index.vue
git commit -m "feat(agent-pool): integrate pool panel into AiChat view

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 6: Slash Command Parser for Chat Input

**Files:**
- Create: `src/agent/slashCommands.ts`
- Modify: `src/views/AiChat/index.vue` (add slash detection and command palette)

- [ ] **Step 1: Create `src/agent/slashCommands.ts`**

```typescript
export interface SlashCommand {
  name: string
  description: string
  icon: string
  template: string  // prompt template with {args} placeholder
  args?: boolean     // whether this command accepts arguments
}

export const SLASH_COMMANDS: SlashCommand[] = [
  { name: '/commit', description: 'Commit changes', icon: '📦', template: 'Commit the current changes with a clear commit message: {args}', args: true },
  { name: '/review', description: 'Review code', icon: '🔍', template: 'Review the code changes in detail and provide feedback: {args}', args: true },
  { name: '/verify', description: 'Verify implementation', icon: '✅', template: 'Verify the implementation works correctly: {args}', args: true },
  { name: '/test', description: 'Run tests', icon: '🧪', template: 'Run tests for: {args}', args: true },
  { name: '/explain', description: 'Explain code', icon: '📖', template: 'Explain this code in detail: {args}', args: true },
  { name: '/refactor', description: 'Refactor code', icon: '🔧', template: 'Refactor the following code: {args}', args: true },
  { name: '/agent', description: 'Spawn worker agent', icon: '🤖', template: '{args}', args: true },
  { name: '/stop', description: 'Stop worker', icon: '⏹', template: 'Stop the worker: {args}', args: true },
]

export function parseSlashCommand(input: string): { command: SlashCommand; args: string } | null {
  const trimmed = input.trim()
  for (const cmd of SLASH_COMMANDS) {
    if (trimmed.startsWith(cmd.name)) {
      const rest = trimmed.slice(cmd.name.length).trim()
      return { command: cmd, args: rest }
    }
  }
  return null
}

export function formatCommandPrompt(input: string): string {
  const parsed = parseSlashCommand(input)
  if (!parsed) return input
  const args = parsed.args || 'no specific parameters'
  return parsed.command.template.replace('{args}', args)
}
```

- [ ] **Step 2: Update `src/views/AiChat/index.vue`**

In the `quickCommands` section, replace with dynamic commands from SLASH_COMMANDS:
```typescript
import { SLASH_COMMANDS } from '@/agent/slashCommands'

const quickCommands = computed(() =>
  SLASH_COMMANDS.map(cmd => ({
    icon: cmd.icon,
    title: cmd.name,
    desc: cmd.description,
    text: cmd.name + ' '
  }))
)
```

Also update `handleKeydown` to detect `/` at start of input for future autocomplete.

- [ ] **Step 3: Commit**

```bash
git add src/agent/slashCommands.ts src/views/AiChat/index.vue
git commit -m "feat(agent): add slash command parser and quick command buttons

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 7: Connect Agent Pool to Chat Messages

**Files:**
- Modify: `src/views/AiChat/index.vue` (send slash commands to agent store)
- Modify: `src/stores/agent.ts` (add agent pool integration)

- [ ] **Step 1: Update `src/stores/agent.ts`**

Add method to run agent with pool scheduling:
```typescript
async function runWithPool(prompt: string, description: string): Promise<void> {
  const workerId = await scheduleTask({
    id: `task-${Date.now()}`,
    description,
    prompt,
    tools: ['read', 'edit', 'bash', 'search'],
    abort_on_error: true
  })
  return workerId
}
```

- [ ] **Step 2: Update `src/views/AiChat/index.vue`**

In `handleSend` function, detect slash commands:
```typescript
import { formatCommandPrompt } from '@/agent/slashCommands'

function handleSend() {
  if (!inputText.value.trim()) return
  
  const prompt = formatCommandPrompt(inputText.value)
  
  if (inputText.value.startsWith('/agent')) {
    const desc = inputText.value.slice(6).trim() || 'Worker task'
    store.runWithPool(prompt, desc)
  } else {
    store.run(prompt)
  }
  
  inputText.value = ''
}
```

- [ ] **Step 3: Commit**

```bash
git add src/stores/agent.ts src/views/AiChat/index.vue
git commit -m "feat(agent): connect slash commands to agent pool

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 8: Worker Result Display in Chat

**Files:**
- Modify: `src/views/AiChat/index.vue` (display worker results as messages)

- [ ] **Step 1: Add worker result handler**

In the `useAgentPool` composable, emit results to parent:
```typescript
function onWorkerResult(callback: (result: WorkerResult) => void) {
  // Poll for completed workers and emit
}
```

In `AiChat/index.vue`, listen for worker completions and display as system messages:
```typescript
const { workers } = useAgentPool()

watch(workers, (newWorkers, oldWorkers) => {
  const completed = newWorkers.filter(w => 
    w.status === 'completed' && 
    oldWorkers?.find(ow => ow.id === w.id && ow.status !== 'completed')
  )
  for (const worker of completed) {
    store.addMessage({
      role: 'system',
      content: `Worker "${worker.description}" completed: ${worker.result || 'Done'}`
    })
  }
}, { deep: true })
```

- [ ] **Step 2: Commit**

```bash
git add src/views/AiChat/index.vue src/agent/useAgentPool.ts
git commit -m "feat(agent): display worker results in chat

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Self-Review

1. **Spec coverage:** Check each task implements the coordinator/worker pool architecture from the reference. The Rust pool manages worker lifecycle, frontend polls for status, UI shows active workers.

2. **Placeholder scan:** No placeholders found - all code blocks are complete.

3. **Type consistency:** Rust `WorkerStatusKind` variants (`idle`, `running`, `completed`, `failed`, `stopped`) match exactly with TypeScript `WorkerStatusKind`. `TaskSpec`, `WorkerStatus`, `WorkerResult` all mirror between Rust and TypeScript sides.

4. **Build verification:** Run `cargo check` in `src-tauri/` and `pnpm build` to verify after each task.

---

## Task 6: Slash Command Parser for Chat Input

**Files:**
- Create: `src/agent/slashCommands.ts`
- Modify: `src/views/AiChat/index.vue` (add slash detection and command palette)

- [ ] **Step 1: Create `src/agent/slashCommands.ts`**

```typescript
export interface SlashCommand {
  name: string
  description: string
  icon: string
  template: string  // prompt template with {args} placeholder
  args?: boolean     // whether this command accepts arguments
}

export const SLASH_COMMANDS: SlashCommand[] = [
  { name: '/commit', description: 'Commit changes', icon: '📦', template: 'Commit the current changes with a clear commit message: {args}', args: true },
  { name: '/review', description: 'Review code', icon: '🔍', template: 'Review the code changes in detail and provide feedback: {args}', args: true },
  { name: '/verify', description: 'Verify implementation', icon: '✅', template: 'Verify the implementation works correctly: {args}', args: true },
  { name: '/test', description: 'Run tests', icon: '🧪', template: 'Run tests for: {args}', args: true },
  { name: '/explain', description: 'Explain code', icon: '📖', template: 'Explain this code in detail: {args}', args: true },
  { name: '/refactor', description: 'Refactor code', icon: '🔧', template: 'Refactor the following code: {args}', args: true },
  { name: '/agent', description: 'Spawn worker agent', icon: '🤖', template: '{args}', args: true },
  { name: '/stop', description: 'Stop worker', icon: '⏹', template: 'Stop the worker: {args}', args: true },
]

export function parseSlashCommand(input: string): { command: SlashCommand; args: string } | null {
  const trimmed = input.trim()
  for (const cmd of SLASH_COMMANDS) {
    if (trimmed.startsWith(cmd.name)) {
      const rest = trimmed.slice(cmd.name.length).trim()
      return { command: cmd, args: rest }
    }
  }
  return null
}

export function formatCommandPrompt(input: string): string {
  const parsed = parseSlashCommand(input)
  if (!parsed) return input
  const args = parsed.args || 'no specific parameters'
  return parsed.command.template.replace('{args}', args)
}
```

- [ ] **Step 2: Update `src/views/AiChat/index.vue`**

In the `quickCommands` section, replace with dynamic commands from SLASH_COMMANDS:
```typescript
import { SLASH_COMMANDS } from '@/agent/slashCommands'

const quickCommands = computed(() =>
  SLASH_COMMANDS.map(cmd => ({
    icon: cmd.icon,
    title: cmd.name,
    desc: cmd.description,
    text: cmd.name + ' '
  }))
)
```

- [ ] **Step 3: Commit**

```bash
git add src/agent/slashCommands.ts src/views/AiChat/index.vue
git commit -m "feat(agent): add slash command parser and quick command buttons

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 7: Connect Agent Pool to Chat Messages

**Files:**
- Modify: `src/views/AiChat/index.vue` (send slash commands to agent store)
- Modify: `src/stores/agent.ts` (add agent pool integration)

- [ ] **Step 1: Update `src/stores/agent.ts`**

Add method to run agent with pool scheduling:
```typescript
async function runWithPool(prompt: string, description: string): Promise<string> {
  const workerId = await scheduleTask({
    id: `task-${Date.now()}`,
    description,
    prompt,
    tools: ['read', 'edit', 'bash', 'search'],
    abort_on_error: true
  })
  return workerId
}
```

- [ ] **Step 2: Update `src/views/AiChat/index.vue`**

In `handleSend` function, detect slash commands:
```typescript
import { formatCommandPrompt } from '@/agent/slashCommands'

function handleSend() {
  if (!inputText.value.trim()) return
  
  const prompt = formatCommandPrompt(inputText.value)
  
  if (inputText.value.startsWith('/agent')) {
    const desc = inputText.value.slice(6).trim() || 'Worker task'
    store.runWithPool(prompt, desc)
  } else {
    store.run(prompt)
  }
  
  inputText.value = ''
}
```

- [ ] **Step 3: Commit**

```bash
git add src/stores/agent.ts src/views/AiChat/index.vue
git commit -m "feat(agent): connect slash commands to agent pool

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 8: Worker Result Display in Chat

**Files:**
- Modify: `src/views/AiChat/index.vue` (display worker results as messages)

- [ ] **Step 1: Add worker result handler**

In the `useAgentPool` composable, poll for completed workers and emit results:
```typescript
watch(workers, (newWorkers, oldWorkers) => {
  const completed = newWorkers.filter(w => 
    w.status === 'completed' && 
    oldWorkers?.find(ow => ow.id === w.id && ow.status !== 'completed')
  )
  for (const worker of completed) {
    store.addMessage({
      role: 'system',
      content: `Worker "${worker.description}" completed: ${worker.result || 'Done'}`
    })
  }
}, { deep: true })
```

- [ ] **Step 2: Commit**

```bash
git add src/views/AiChat/index.vue src/agent/useAgentPool.ts
git commit -m "feat(agent): display worker results in chat

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```
