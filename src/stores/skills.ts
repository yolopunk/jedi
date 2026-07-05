// src/stores/skills.ts

import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { skillRegistry } from '@/skills/registry'

export const useSkillsStore = defineStore('skills', () => {
  const enabledSkills = ref<string[]>([
    'terminal',
    'filesystem',
    'hosts',
    'web_search',
    'web_fetch',
    'memory',
  ])
  const skillPanelOpen = ref(false)

  const allSkills = computed(() => skillRegistry.list())
  const enabledSkillsList = computed(() => skillRegistry.listEnabled())
  const autoCallableSkills = computed(() => skillRegistry.listAutoCallable())

  function toggleSkill(id: string, enabled: boolean): void {
    skillRegistry.setEnabled(id, enabled)
    if (enabled) {
      if (!enabledSkills.value.includes(id)) enabledSkills.value.push(id)
    } else {
      enabledSkills.value = enabledSkills.value.filter(s => s !== id)
    }
    saveToStorage()
  }

  function isSkillEnabled(id: string): boolean {
    return skillRegistry.get(id)?.enabled ?? false
  }

  function toggleSkillPanel(): void {
    skillPanelOpen.value = !skillPanelOpen.value
  }

  function setSkillPanelOpen(open: boolean): void {
    skillPanelOpen.value = open
  }

  function saveToStorage(): void {
    try {
      localStorage.setItem('skills-enabled', JSON.stringify(enabledSkills.value))
    } catch (e) {
      console.error('Failed to save skills:', e)
    }
  }

  function loadFromStorage(): void {
    try {
      const saved = localStorage.getItem('skills-enabled')
      if (saved) {
        const ids = JSON.parse(saved) as string[]
        enabledSkills.value = ids
        ids.forEach(id => {
          skillRegistry.setEnabled(id, true)
        })
      }
    } catch (e) {
      console.error('Failed to load skills:', e)
    }
  }

  return {
    enabledSkills,
    skillPanelOpen,
    allSkills,
    enabledSkillsList,
    autoCallableSkills,
    toggleSkill,
    isSkillEnabled,
    toggleSkillPanel,
    setSkillPanelOpen,
    loadFromStorage,
  }
})
