// src/stores/skills.ts
//
// Skill management store. Owns two concerns:
//  1. Persisted per-skill flags (enabled / autoCallable) — stored via
//     useStorage (Tauri Store settings.json) and applied to the registry
//     in BOTH directions, so disabling a skill survives restarts (the old
//     localStorage 'skills-enabled' scheme only re-enabled and is migrated
//     away on first load).
//  2. Custom markdown skills from ~/.jedi/skills/ — loaded via backend
//     commands and bridged into the same skillRegistry.

import { deleteCustomSkill, listCustomSkills, saveCustomSkill } from '@/api/skills'
import { useStorage } from '@/composables/useStorage'
import { createCustomSkill } from '@/skills/custom'
import { skillRegistry } from '@/skills/registry'
import type { CustomSkillDef, SkillSource } from '@/skills/types'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

/** 单个技能的持久化开关配置 */
export interface SkillFlags {
  enabled: boolean
  autoCallable: boolean
}

type SkillFlagMap = Record<string, SkillFlags>

const CONFIG_KEY = 'skills-config'
const LEGACY_KEY = 'skills-enabled'

export const useSkillsStore = defineStore('skills', () => {
  const { getItem, setItem } = useStorage()

  const flagMap = ref<SkillFlagMap>({})
  const customDefs = ref<CustomSkillDef[]>([])
  const loaded = ref(false)
  const error = ref<string | null>(null)

  const allSkills = computed(() => skillRegistry.list())
  const enabledSkillsList = computed(() => skillRegistry.listEnabled())
  const autoCallableSkills = computed(() => skillRegistry.listAutoCallable())
  const enabledCount = computed(() => enabledSkillsList.value.length)

  function skillsBySource(source: SkillSource) {
    return allSkills.value.filter(s => s.source === source)
  }

  function isSkillEnabled(id: string): boolean {
    return skillRegistry.get(id)?.enabled ?? false
  }

  function isSkillAutoCallable(id: string): boolean {
    return skillRegistry.get(id)?.autoCallable ?? false
  }

  /** 把持久化配置应用到 registry（有配置的技能双向覆盖，无配置的沿用默认） */
  function syncRegistryFromConfig(): void {
    for (const skill of skillRegistry.list()) {
      const flags = flagMap.value[skill.id]
      if (!flags) continue
      skillRegistry.setEnabled(skill.id, flags.enabled)
      skillRegistry.setAutoCallable(skill.id, flags.autoCallable)
    }
  }

  function persistFlags(id: string): void {
    const skill = skillRegistry.get(id)
    if (!skill) return
    flagMap.value[id] = { enabled: skill.enabled, autoCallable: skill.autoCallable }
    void setItem(CONFIG_KEY, flagMap.value)
  }

  function toggleSkill(id: string, enabled: boolean): void {
    skillRegistry.setEnabled(id, enabled)
    persistFlags(id)
  }

  function toggleAutoCallable(id: string, value: boolean): void {
    skillRegistry.setAutoCallable(id, value)
    persistFlags(id)
  }

  /** 旧版 localStorage 'skills-enabled'（仅启用名单）一次性迁移为双向配置 */
  async function migrateLegacyFlags(): Promise<SkillFlagMap> {
    const existing = (await getItem<SkillFlagMap>(CONFIG_KEY)) ?? {}
    const legacy = localStorage.getItem(LEGACY_KEY)
    if (!legacy) return existing
    try {
      const enabledIds = JSON.parse(legacy) as string[]
      for (const skill of skillRegistry.list()) {
        if (existing[skill.id]) continue
        existing[skill.id] = {
          enabled: enabledIds.includes(skill.id),
          autoCallable: skill.autoCallable,
        }
      }
      await setItem(CONFIG_KEY, existing)
      localStorage.removeItem(LEGACY_KEY)
    } catch (e) {
      console.error('Failed to migrate legacy skills config:', e)
    }
    return existing
  }

  /** 启动时加载：迁移旧数据 → 读配置 → 应用到 registry → 加载自定义技能 */
  async function loadConfig(): Promise<void> {
    if (loaded.value) return
    loaded.value = true
    error.value = null
    try {
      flagMap.value = await migrateLegacyFlags()
      syncRegistryFromConfig()
      await loadCustomSkills()
    } catch (e) {
      console.error('Failed to load skills config:', e)
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  /** 拉取 ~/.jedi/skills/ 下的自定义技能并重建 registry 中的 custom 条目 */
  async function loadCustomSkills(): Promise<void> {
    try {
      const defs = await listCustomSkills()
      customDefs.value = defs
      for (const skill of skillRegistry.list()) {
        if (skill.source === 'custom') skillRegistry.unregister(skill.id)
      }
      for (const def of defs) skillRegistry.register(createCustomSkill(def))
      syncRegistryFromConfig()
    } catch (e) {
      console.error('Failed to load custom skills:', e)
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  /** 新建或更新一个自定义技能（落盘后同步 registry） */
  async function saveCustom(def: CustomSkillDef): Promise<CustomSkillDef> {
    const saved = await saveCustomSkill(def)
    await loadCustomSkills()
    return saved
  }

  /** 删除一个自定义技能（落盘 + 清配置 + 同步 registry） */
  async function removeCustom(id: string): Promise<void> {
    await deleteCustomSkill(id)
    if (flagMap.value[id]) {
      delete flagMap.value[id]
      await setItem(CONFIG_KEY, flagMap.value)
    }
    skillRegistry.unregister(id)
    await loadCustomSkills()
  }

  return {
    customDefs,
    loaded,
    error,
    allSkills,
    enabledSkillsList,
    autoCallableSkills,
    enabledCount,
    skillsBySource,
    isSkillEnabled,
    isSkillAutoCallable,
    toggleSkill,
    toggleAutoCallable,
    syncRegistryFromConfig,
    loadConfig,
    loadCustomSkills,
    saveCustom,
    removeCustom,
  }
})
