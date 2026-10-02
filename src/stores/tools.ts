// src/stores/tools.ts
//
// Tool management store. Owns the persisted per-tool flags (enabled /
// always-allow whitelist) applied to the toolRegistry in BOTH directions so
// they survive restarts. Populated by stores/skills.ts's legacy migration
// before loadToolsConfig() runs.

import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { toolRegistry } from '@/agent/tools/registry'
import { logSecurityEvent } from '@/api/ai-chat'
import { useStorage } from '@/composables/useStorage'

/** 单个工具的持久化开关配置 */
export interface ToolFlags {
  enabled: boolean
  /** write/system 工具的"始终允许"白名单标志，read 工具无意义 */
  alwaysAllow?: boolean
}

type ToolFlagMap = Record<string, ToolFlags>

const CONFIG_KEY = 'tools-config'

export const useToolsStore = defineStore('tools', () => {
  const { getItem, setItem } = useStorage()

  const flagMap = ref<ToolFlagMap>({})
  const loaded = ref(false)

  const allTools = computed(() => toolRegistry.list())
  const enabledToolsList = computed(() => toolRegistry.listEnabled())
  const enabledCount = computed(() => enabledToolsList.value.length)

  function isToolEnabled(id: string): boolean {
    return toolRegistry.get(id)?.enabled ?? false
  }

  /** 把持久化配置应用到 registry（有配置的工具双向覆盖，无配置的沿用默认） */
  function syncRegistryFromConfig(): void {
    for (const tool of toolRegistry.list()) {
      const flags = flagMap.value[tool.id]
      if (!flags) continue
      toolRegistry.setEnabled(tool.id, flags.enabled)
    }
  }

  function persistFlags(id: string): void {
    const tool = toolRegistry.get(id)
    if (!tool) return
    // 展开保留 alwaysAllow 等不在本函数管理内的标志
    flagMap.value[id] = {
      ...flagMap.value[id],
      enabled: tool.enabled,
    }
    void setItem(CONFIG_KEY, flagMap.value)
  }

  function toggleTool(id: string, enabled: boolean): void {
    toolRegistry.setEnabled(id, enabled)
    persistFlags(id)
  }

  /** 该工具是否在"始终允许"白名单内（write/system 确认门免弹卡） */
  function isAlwaysAllowed(id: string): boolean {
    return flagMap.value[id]?.alwaysAllow ?? false
  }

  /** 白名单变更写入审计日志（fire-and-forget，失败不阻塞主流程） */
  function auditWhitelistChange(id: string, value: boolean): void {
    logSecurityEvent({
      event_type: 'skill_whitelist_change',
      result: value ? 'granted' : 'revoked',
      resource: id,
      action: value ? 'always-allow' : 'revoke-always-allow',
    }).catch(e => console.error('Failed to log whitelist change:', e))
  }

  function setAlwaysAllowed(id: string, value: boolean): void {
    if (isAlwaysAllowed(id) === value) return
    flagMap.value[id] = {
      ...flagMap.value[id],
      enabled: toolRegistry.get(id)?.enabled ?? true,
      alwaysAllow: value,
    }
    void setItem(CONFIG_KEY, flagMap.value)
    auditWhitelistChange(id, value)
  }

  /** 启动时加载工具开关配置并应用到 registry（幂等） */
  async function loadToolsConfig(): Promise<void> {
    if (loaded.value) return
    loaded.value = true
    try {
      flagMap.value = (await getItem<ToolFlagMap>(CONFIG_KEY)) ?? {}
      syncRegistryFromConfig()
    } catch (e) {
      console.error('Failed to load tools config:', e)
    }
  }

  /** 供迁移写入拆分后的配置（stores/skills.ts 的 legacy 拆分调用） */
  async function importMigratedFlags(flags: ToolFlagMap): Promise<void> {
    flagMap.value = { ...flags, ...flagMap.value }
    await setItem(CONFIG_KEY, flagMap.value)
    syncRegistryFromConfig()
  }

  return {
    loaded,
    allTools,
    enabledToolsList,
    enabledCount,
    isToolEnabled,
    isAlwaysAllowed,
    setAlwaysAllowed,
    toggleTool,
    syncRegistryFromConfig,
    loadToolsConfig,
    importMigratedFlags,
  }
})
