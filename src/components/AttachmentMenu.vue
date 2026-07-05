<template>
  <div class="attachment-menu">
    <div class="menu-title">Skills · 启用后 agent 可调用</div>
    <div
      v-for="skill in skillsStore.allSkills"
      :key="skill.id"
      class="submenu-item"
      :class="{ enabled: skillsStore.isSkillEnabled(skill.id) }"
      @click.stop="handleSkillClick(skill)"
    >
      <span class="skill-icon">{{ skill.icon }}</span>
      <span class="skill-name">{{ skill.name }}</span>
      <span class="skill-badge" :class="{ on: skillsStore.isSkillEnabled(skill.id) }">
        {{ skillsStore.isSkillEnabled(skill.id) ? 'ON' : 'OFF' }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Skill } from '@/skills/types'
import { useSkillsStore } from '@/stores/skills'

defineEmits<{
  (e: 'close'): void
  (e: 'select', action: string): void
}>()

const skillsStore = useSkillsStore()

function handleSkillClick(skill: Skill) {
  // Toggle the skill; keep the menu open so several can be flipped at once.
  const isEnabled = skillsStore.isSkillEnabled(skill.id)
  skillsStore.toggleSkill(skill.id, !isEnabled)
}
</script>

<style scoped>
.attachment-menu {
  position: absolute;
  bottom: 100%;
  left: 0;
  margin-bottom: 8px;
  min-width: 200px;
  background: var(--bg-terminal);
  border: 1px solid rgb(var(--text-rgb) / 0.1);
  border-radius: 10px;
  padding: 6px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  z-index: 100;
}

.menu-title {
  padding: 6px 10px 8px;
  font-size: 11px;
  color: rgb(var(--text-rgb) / 0.4);
}

.submenu-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 6px;
  cursor: pointer;
  color: rgb(var(--text-rgb) / 0.7);
  font-size: 13px;
  transition: all 0.1s;
}

.submenu-item:hover {
  background:rgb(var(--ink-rgb) / 0.05);
  color: var(--text);
}

.submenu-item.enabled {
  color: var(--success);
}

.submenu-item .skill-icon {
  font-size: 14px;
}

.submenu-item .skill-name {
  flex: 1;
}

.submenu-item .skill-badge {
  font-size: 9px;
  padding: 2px 6px;
  background:rgb(var(--ink-rgb) / 0.08);
  color: rgb(var(--text-rgb) / 0.4);
  border-radius: 4px;
  font-weight: 700;
}

.submenu-item .skill-badge.on {
  background: var(--success);
  color: var(--text-inverse);
}
</style>
