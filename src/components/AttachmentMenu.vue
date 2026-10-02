<template>
  <div class="attachment-menu" role="menu" :aria-label="$t('skills.menuTitle')">
    <div class="menu-title">{{ $t('skills.menuTitle') }}</div>
    <div
      v-for="skill in skillsStore.allSkills"
      :key="skill.name"
      class="submenu-item"
      :class="{ enabled: skillsStore.isSkillEnabled(skill.name) }"
      role="menuitemcheckbox"
      :aria-checked="skillsStore.isSkillEnabled(skill.name)"
      tabindex="0"
      @click.stop="handleSkillClick(skill)"
      @keydown.enter.prevent="handleSkillClick(skill)"
      @keydown.space.prevent="handleSkillClick(skill)"
    >
      <span class="skill-icon" aria-hidden="true">{{ skill.icon }}</span>
      <span class="skill-name">/{{ skill.name }}</span>
      <span class="skill-badge" :class="{ on: skillsStore.isSkillEnabled(skill.name) }">
        {{ skillsStore.isSkillEnabled(skill.name) ? 'ON' : 'OFF' }}
      </span>
    </div>
    <div
      class="manage-item"
      role="menuitem"
      tabindex="0"
      @click.stop="emit('manage')"
      @keydown.enter.prevent="emit('manage')"
    >
      <span class="manage-icon" aria-hidden="true">⚙</span>
      <span class="manage-text">{{ $t('skills.manageEntry') }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { SkillManifest } from '@/skills/types'
import { useSkillsStore } from '@/stores/skills'

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'manage'): void
}>()

const skillsStore = useSkillsStore()

function handleSkillClick(skill: SkillManifest) {
  // Toggle the skill; keep the menu open so several can be flipped at once.
  const isEnabled = skillsStore.isSkillEnabled(skill.name)
  skillsStore.toggleSkill(skill.name, !isEnabled)
}
</script>

<style scoped>
.attachment-menu {
  position: absolute;
  bottom: 100%;
  left: 0;
  margin-bottom: 8px;
  min-width: 200px;
  max-height: 320px;
  overflow: auto;
  background: var(--bg-terminal);
  border: 1px solid rgb(var(--text-rgb) / 0.1);
  border-radius: 10px;
  padding: 6px;
  box-shadow: var(--shadow-lg);
  z-index: 100;
}

.menu-title {
  padding: 6px 10px 8px;
  font-size: 11px;
  color: var(--text-muted);
}

.submenu-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 6px;
  cursor: pointer;
  color: var(--text-muted);
  font-size: 13px;
  transition: all 0.1s;
}

.submenu-item:hover,
.submenu-item:focus-visible {
  background: rgb(var(--ink-rgb) / 0.05);
  color: var(--text);
  outline: none;
}

.submenu-item.enabled {
  color: var(--success);
}

.submenu-item .skill-icon {
  font-size: 14px;
}

.submenu-item .skill-name {
  flex: 1;
  font-family: var(--jedi-font-mono);
  font-size: 12px;
}

.submenu-item .skill-badge {
  font-size: 9px;
  padding: 2px 6px;
  background: rgb(var(--ink-rgb) / 0.08);
  color: var(--text-muted);
  border-radius: 4px;
  font-weight: 700;
}

.submenu-item .skill-badge.on {
  background: var(--success);
  color: var(--text-inverse);
}

.manage-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  margin-top: 4px;
  border-top: 1px solid rgb(var(--text-rgb) / 0.08);
  border-radius: 0 0 6px 6px;
  cursor: pointer;
  color: var(--text-muted);
  font-size: 12px;
  transition: all 0.1s;
}

.manage-item:hover,
.manage-item:focus-visible {
  color: var(--text);
  outline: none;
}
</style>
