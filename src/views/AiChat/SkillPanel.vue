<!-- src/views/AiChat/SkillPanel.vue -->
<template>
  <div class="skill-panel">
    <div class="panel-header">
      <span class="panel-title">// SKILLS</span>
      <span class="panel-status">READY</span>
    </div>
    <div class="skills-list">
      <div
        v-for="skill in store.allSkills"
        :key="skill.id"
        class="skill-item"
        :class="{ active: store.isSkillEnabled(skill.id) }"
        @click="toggleSkill(skill.id)"
      >
        <div class="skill-indicator" :class="{ enabled: store.isSkillEnabled(skill.id) }"></div>
        <span class="skill-icon">{{ skill.icon }}</span>
        <span class="skill-name">{{ skill.name }}</span>
      </div>
    </div>
    <div class="panel-footer">
      <span class="footer-text">CLICK TO TOGGLE</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useSkillsStore } from '@/stores/skills'

const store = useSkillsStore()

function toggleSkill(id: string): void {
  const isEnabled = store.isSkillEnabled(id)
  store.toggleSkill(id, !isEnabled)
}
</script>

<style scoped>
.skill-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background:rgb(var(--ink-rgb) / 0.3);
  border-right: 1px solid rgb(var(--accent-rgb) / 0.2);
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid rgb(var(--accent-rgb) / 0.2);
  background: rgb(var(--accent-rgb) / 0.05);
}

.panel-title {
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
  font-size: 12px;
  font-weight: 700;
  color: var(--accent);
  letter-spacing: 1px;
}

.panel-status {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  color: var(--success);
}

.skills-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.skill-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
  border: 1px solid transparent;
  margin-bottom: 4px;
}

.skill-item:hover {
  background: rgb(var(--accent-rgb) / 0.05);
  border-color: rgb(var(--accent-rgb) / 0.2);
}

.skill-item.active {
  background: rgb(var(--success-rgb) / 0.1);
  border-color: rgb(var(--success-rgb) / 0.4);
}

.skill-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-subtle);
}

.skill-indicator.enabled {
  background: var(--success);
  box-shadow: 0 0 8px var(--success);
}

.skill-icon {
  font-size: 16px;
  width: 24px;
  text-align: center;
}

.skill-name {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 600;
  color: var(--border);
  letter-spacing: 0.5px;
}

.panel-footer {
  padding: 12px 16px;
  border-top: 1px solid rgb(var(--accent-rgb) / 0.1);
  background:rgb(var(--ink-rgb) / 0.2);
}

.footer-text {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  color: var(--text-subtle);
  letter-spacing: 1px;
}

/* Scrollbar */
.skills-list::-webkit-scrollbar {
  width: 6px;
}

.skills-list::-webkit-scrollbar-track {
  background: transparent;
}

.skills-list::-webkit-scrollbar-thumb {
  background: rgb(var(--accent-rgb) / 0.2);
  border-radius: 3px;
}
</style>
