<template>
  <div class="skills-page">
    <!-- Console 工具栏 -->
    <div class="console-header-bar">
      <span class="bar-title">[SKILLS]</span>
      <span class="bar-count">
        {{ $t('skills.enabledCount', { count: toolsStore.enabledCount + skillsStore.enabledCount }) }}
      </span>
    </div>

    <SkillsManagerContent show-search />
  </div>
</template>

<script setup lang="ts">
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
  gap: 14px;
  padding: 16px 18px;
  overflow: auto;
  color: var(--text);
}

.console-header-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border);
}

.bar-title {
  font-family: var(--jedi-font-mono);
  font-size: 15px;
  font-weight: 700;
  letter-spacing: 0.5px;
  color: var(--accent);
}

.bar-count {
  font-family: var(--jedi-font-mono);
  font-size: 11px;
  color: var(--text-muted);
}
</style>
