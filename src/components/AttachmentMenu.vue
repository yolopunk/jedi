<template>
  <div class="attachment-menu">
    <div class="menu-item" @click="$emit('select', 'attachment')">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
        <path d="M21.44 11.05l-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48" stroke="currentColor" stroke-width="2"/>
      </svg>
      <span>Attachment</span>
    </div>

    <!-- Skills 子菜单 -->
    <div
      class="menu-item has-submenu"
      :class="{ expanded: showSkillsSubmenu }"
      @click.stop="showSkillsSubmenu = !showSkillsSubmenu"
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
        <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" stroke="currentColor" stroke-width="2"/>
      </svg>
      <span>Skills</span>
      <svg class="arrow" width="12" height="12" viewBox="0 0 24 24" fill="none">
        <polyline points="9 18 15 12 9 6" stroke="currentColor" stroke-width="2"/>
      </svg>

      <!-- Skills 子菜单 -->
      <div v-if="showSkillsSubmenu" class="skills-submenu" @click.stop>
        <div
          v-for="skill in skillsStore.allSkills"
          :key="skill.id"
          class="submenu-item"
          :class="{ enabled: skillsStore.isSkillEnabled(skill.id) }"
          @click="handleSkillClick(skill)"
        >
          <span class="skill-icon">{{ skill.icon }}</span>
          <span class="skill-name">{{ skill.name }}</span>
          <span v-if="skillsStore.isSkillEnabled(skill.id)" class="skill-badge">ON</span>
        </div>
      </div>
    </div>

    <div class="menu-item" @click="$emit('select', 'web-search')">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
        <circle cx="11" cy="11" r="8" stroke="currentColor" stroke-width="2"/>
        <line x1="21" y1="21" x2="16.65" y2="16.65" stroke="currentColor" stroke-width="2"/>
      </svg>
      <span>Web Search</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import type { Skill } from '@/skills/types'
import { useSkillsStore } from '@/stores/skills'

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'select', action: string): void
}>()

const skillsStore = useSkillsStore()
const showSkillsSubmenu = ref(false)

function handleSkillClick(skill: Skill) {
  // Toggle skill enabled state
  const isEnabled = skillsStore.isSkillEnabled(skill.id)
  skillsStore.toggleSkill(skill.id, !isEnabled)
  // Don't close the menu so user can toggle multiple skills
}
</script>

<style scoped>
.attachment-menu {
  position: absolute;
  bottom: 100%;
  left: 0;
  margin-bottom: 8px;
  min-width: 160px;
  background: #0d1117;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  padding: 6px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  z-index: 100;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 6px;
  cursor: pointer;
  color: rgba(255, 255, 255, 0.7);
  font-size: 13px;
  transition: all 0.1s;
}

.menu-item:hover {
  background: rgba(255, 255, 255, 0.05);
  color: #ffffff;
}

.menu-item.has-submenu {
  position: relative;
}

.menu-item.has-submenu .arrow {
  margin-left: auto;
  transition: transform 0.2s;
}

.menu-item.has-submenu.expanded .arrow {
  transform: rotate(90deg);
}

.skills-submenu {
  position: absolute;
  left: 100%;
  top: 0;
  margin-left: 4px;
  min-width: 180px;
  background: #0d1117;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  padding: 6px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  z-index: 101;
}

.submenu-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 6px;
  cursor: pointer;
  color: rgba(255, 255, 255, 0.7);
  font-size: 13px;
  transition: all 0.1s;
}

.submenu-item:hover {
  background: rgba(255, 255, 255, 0.05);
  color: #ffffff;
}

.submenu-item.enabled {
  background: rgba(0, 255, 136, 0.08);
  color: #00ff88;
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
  background: #00ff88;
  color: #000;
  border-radius: 4px;
  font-weight: 700;
}
</style>
