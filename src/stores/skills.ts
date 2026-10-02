// src/stores/skills.ts
//
// Skill management store. Skills are SKILL.md instruction packages (no
// executable body — tools live in stores/tools.ts). This store owns:
//  1. The skill manifests (builtin, registered at module load; user skills
//     pulled from ~/.jedi/skills/ via backend commands) mirrored into
//     skillRegistry.
//  2. The persisted per-skill enabled flag ('skills-flags' storage key).
//  3. The one-time legacy migration: the old combined 'skills-config'
//     ({id: {enabled, autoCallable, alwaysAllow}}) is split — tool entries
//     move into the tools store ('tools-config'), the rest is dropped.

import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { toolRegistry } from '@/agent/tools/registry'
import type { SkillSaveDef } from '@/api/skills'
import { deleteSkill, listSkills, saveSkill } from '@/api/skills'
import { useStorage } from '@/composables/useStorage'
import { invalidateSkill } from '@/skills/loader'
import { skillRegistry } from '@/skills/registry'
import type { SkillManifest, SkillSource } from '@/skills/types'
import { type ToolFlags, useToolsStore } from './tools'

const CONFIG_KEY = 'skills-flags'
const LEGACY_KEY = 'skills-config'

export const useSkillsStore = defineStore('skills', () => {
  const { getItem, setItem, removeItem } = useStorage()

  const flagMap = ref<Record<string, { enabled: boolean }>>({})
  const userSkills = ref<SkillManifest[]>([])
  const loaded = ref(false)
  const error = ref<string | null>(null)

  const allSkills = computed(() => skillRegistry.list())
  const enabledSkillsList = computed(() => skillRegistry.listEnabled())
  const enabledCount = computed(() => enabledSkillsList.value.length)

  function skillsBySource(source: SkillSource) {
    return allSkills.value.filter(s => s.source === source)
  }

  function isSkillEnabled(name: string): boolean {
    return skillRegistry.get(name)?.enabled ?? false
  }

  /** 把持久化配置应用到 registry（有配置的技能双向覆盖，无配置的沿用默认） */
  function syncRegistryFromConfig(): void {
    for (const skill of skillRegistry.list()) {
      const flags = flagMap.value[skill.name]
      if (!flags) continue
      skillRegistry.setEnabled(skill.name, flags.enabled)
    }
  }

  function persistFlags(name: string): void {
    const skill = skillRegistry.get(name)
    if (!skill) return
    flagMap.value[name] = { enabled: skill.enabled }
    void setItem(CONFIG_KEY, flagMap.value)
  }

  function toggleSkill(name: string, enabled: boolean): void {
    skillRegistry.setEnabled(name, enabled)
    persistFlags(name)
  }

  /**
   * 一次性迁移：旧版合并配置 'skills-config' 拆分为 tools-config（工具开关 +
   * 白名单）与 skills-flags（技能开关）。工具 id 以 toolRegistry 当前注册项
   * 为准，无法识别的条目（旧自定义技能等）直接丢弃。
   */
  async function migrateLegacyConfig(): Promise<void> {
    const legacy = await getItem<Record<string, ToolFlags>>(LEGACY_KEY)
    if (!legacy) return
    const toolFlags: Record<string, ToolFlags> = {}
    for (const [id, flags] of Object.entries(legacy)) {
      if (toolRegistry.get(id)) toolFlags[id] = flags
    }
    if (Object.keys(toolFlags).length > 0) {
      await useToolsStore().importMigratedFlags(toolFlags)
    }
    await removeItem(LEGACY_KEY)
  }

  /** 启动时加载：迁移旧数据 → 读配置 → 拉取用户技能 → 应用到 registry → 加载工具配置 */
  async function loadConfig(): Promise<void> {
    if (loaded.value) return
    loaded.value = true
    error.value = null
    try {
      await migrateLegacyConfig()
      flagMap.value = (await getItem<Record<string, { enabled: boolean }>>(CONFIG_KEY)) ?? {}
      await loadUserSkills()
      syncRegistryFromConfig()
      await useToolsStore().loadToolsConfig()
    } catch (e) {
      console.error('Failed to load skills config:', e)
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  /** 拉取 ~/.jedi/skills/ 下的用户技能并重建 registry 中的 user 条目 */
  async function loadUserSkills(): Promise<void> {
    try {
      const manifests = await listSkills()
      userSkills.value = manifests.map(m => ({ ...m, source: 'user' as const, enabled: true }))
      for (const skill of skillRegistry.list()) {
        if (skill.source === 'user') skillRegistry.unregister(skill.name)
      }
      for (const manifest of userSkills.value) skillRegistry.register(manifest)
    } catch (e) {
      console.error('Failed to load user skills:', e)
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  /** 新建或更新一个用户技能（落盘后同步 registry 与正文缓存） */
  async function saveUserSkill(def: SkillSaveDef): Promise<void> {
    await saveSkill(def)
    invalidateSkill(def.name)
    await loadUserSkills()
    syncRegistryFromConfig()
  }

  /** 删除一个用户技能（落盘 + 清配置 + 同步 registry） */
  async function removeUserSkill(name: string): Promise<void> {
    await deleteSkill(name)
    invalidateSkill(name)
    if (flagMap.value[name]) {
      delete flagMap.value[name]
      await setItem(CONFIG_KEY, flagMap.value)
    }
    skillRegistry.unregister(name)
    await loadUserSkills()
  }

  return {
    userSkills,
    loaded,
    error,
    allSkills,
    enabledSkillsList,
    enabledCount,
    skillsBySource,
    isSkillEnabled,
    toggleSkill,
    syncRegistryFromConfig,
    loadConfig,
    loadUserSkills,
    saveUserSkill,
    removeUserSkill,
  }
})
