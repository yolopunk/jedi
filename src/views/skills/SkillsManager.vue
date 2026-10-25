<template>
  <div class="skills-page">
    <!-- 页头：与聊天/Hosts 页头统一 -->
    <div class="skills-header">
      <div class="header-logo">
        <v-icon :icon="mdiTools" size="18" aria-hidden="true" />
        <span class="header-title">{{ $t('sidebar.skills') }}</span>
      </div>
      <span class="header-count">
        {{ $t('skills.enabledCount', { count: toolsStore.enabledCount + skillsStore.enabledCount }) }}
      </span>
    </div>

    <div class="skills-content">
      <SkillsManagerContent show-search />
    </div>
  </div>
</template>

<script setup lang="ts">
import { mdiTools } from '@mdi/js'
import SkillsManagerContent from '@/components/skills/SkillsManagerContent.vue'
import { useSkillsStore } from '@/stores/skills'
import { useToolsStore } from '@/stores/tools'

const skillsStore = useSkillsStore()
const toolsStore = useToolsStore()
</script>

<style scoped>
.skills-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  color: var(--text);
}

.skills-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 10px;
  flex-shrink: 0;
  min-height: 60px;
  padding: 12px 24px;
  background: var(--bg);
  border-bottom: 1px solid var(--border);
}

.skills-header .header-logo {
  display: flex;
  align-items: center;
  gap: 10px;
  color: var(--text);
}

.skills-header .header-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text);
  font-family: var(--jedi-font-ui);
}

.skills-header .header-count {
  font-family: var(--jedi-font-mono);
  font-size: 11px;
  color: var(--text-muted);
}

.skills-content {
  flex: 1;
  min-height: 0;
  overflow: auto;
  display: flex;
  flex-direction: column;
  padding: 16px 18px;
}
</style>
