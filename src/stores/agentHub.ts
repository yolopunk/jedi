// Agent hub store (docs/ai-chat/05-multi-agent-rd-plan.md §3.1 / §5.4).
//
// Owns detection state from the Rust AgentDetector and derives the selectable
// engine list ("agent@host" entries + the builtin engine). The selected
// engine persists across restarts; external engines without a driver (M2/M3)
// stay visible but the chat falls back to builtin until their driver lands.

import { invoke } from '@tauri-apps/api/core'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import type { AgentReport, DetectionResult, EngineDescriptor } from '@/agent/hub/types'
import { useStorage } from '@/composables/useStorage'

/** Driver tier per docs §3.2: A = adapter shipped, B = pending verification. */
const TIER_A_AGENTS = new Set([
  'claude-code',
  'codex',
  'pi',
  'gemini-cli',
  'qwen-code',
  'cursor-cli',
  'opencode',
  'antigravity',
])

const ENGINE_KEY = 'agent-hub-engine'

function locationBadge(host: string): string {
  if (host === 'windows') return 'Windows'
  if (host === 'macos') return 'macOS'
  if (host === 'linux') return 'Linux'
  if (host.startsWith('wsl:')) return `WSL:${host.slice(4)}`
  return host
}

export const useAgentHubStore = defineStore('agentHub', () => {
  const { getItem, setItem } = useStorage()

  const reports = ref<AgentReport[]>([])
  const scanning = ref(false)
  const error = ref<string | null>(null)
  const detectedAt = ref<number | null>(null)

  const builtinEngine: EngineDescriptor = {
    id: 'builtin',
    agentId: 'builtin',
    name: 'Jedi 内置',
    location: '',
    host: 'builtin',
    tier: 'builtin',
  }

  /** All selectable engines: builtin first, then every detected installation. */
  const engines = computed<EngineDescriptor[]>(() => {
    const external: EngineDescriptor[] = []
    for (const report of reports.value) {
      for (const inst of report.installations) {
        external.push({
          id: `${inst.agentId}@${inst.host}`,
          agentId: report.id,
          name: report.name,
          location: locationBadge(inst.host),
          host: inst.host,
          installation: inst,
          tier: TIER_A_AGENTS.has(report.id) ? 'A' : 'B',
        })
      }
    }
    return [builtinEngine, ...external]
  })

  const selectedEngineId = ref<string>('builtin')
  const selectedEngine = computed<EngineDescriptor>(
    () => engines.value.find(e => e.id === selectedEngineId.value) ?? builtinEngine
  )
  /** Detected-but-driverless engines (shown greyed with a "coming soon" tag). */
  const pendingDrivers = computed(() =>
    engines.value.filter(e => e.tier !== 'builtin' && e.tier !== 'A')
  )

  async function detect(force = false): Promise<void> {
    if (scanning.value) return
    scanning.value = true
    error.value = null
    try {
      const result = await invoke<DetectionResult>('detect_agents', { force: force || null })
      reports.value = result.reports
      // Drop a persisted selection whose engine vanished.
      const ids = new Set(engines.value.map(e => e.id))
      if (!ids.has(selectedEngineId.value)) selectedEngineId.value = 'builtin'
    } catch (e: any) {
      error.value = e?.message ?? String(e)
    } finally {
      scanning.value = false
      detectedAt.value = Date.now()
    }
  }

  function selectEngine(id: string): void {
    const engine = engines.value.find(e => e.id === id)
    if (!engine) return
    selectedEngineId.value = id
    void setItem(ENGINE_KEY, id)
  }

  async function loadPersisted(): Promise<void> {
    try {
      const saved = await getItem<string>(ENGINE_KEY)
      if (saved) selectedEngineId.value = saved
    } catch {
      /* keep builtin */
    }
  }

  const installedCount = computed(() => engines.value.filter(e => e.tier !== 'builtin').length)

  return {
    reports,
    scanning,
    error,
    detectedAt,
    engines,
    builtinEngine,
    selectedEngineId,
    selectedEngine,
    pendingDrivers,
    installedCount,
    detect,
    selectEngine,
    loadPersisted,
  }
})
